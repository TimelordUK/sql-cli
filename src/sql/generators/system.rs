//! System tables: the running machine as something to query (S1, S2).
//!
//! `SELECT * FROM processes() WHERE name LIKE 'node%'`,
//! `SELECT * FROM sockets() WHERE state = 'LISTEN'`.
//!
//! These are ordinary [`TableGenerator`]s, the same shape as the file readers
//! next door, so nothing in the parser or the executor knows they are special.
//! Joining them to each other is what CTEs are for:
//!
//! ```sql
//! WITH p AS (SELECT * FROM processes())
//! SELECT name, memory_bytes FROM p ORDER BY memory_bytes DESC LIMIT 10
//! ```
//!
//! **The columns are the same on every platform.** Where an OS cannot answer
//! - or will not, without privileges it has not been given - the cell is NULL
//! rather than the table being a different shape on Windows than on Linux. A
//! query written on one machine has to run on the other; that is the whole
//! point of normalising here rather than exposing what each OS happens to
//! offer.

use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use crate::sql::generators::TableGenerator;
use anyhow::Result;
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo, TcpState,
};
use std::net::IpAddr;
use std::sync::Arc;
use sysinfo::{
    Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, System, Users,
    MINIMUM_CPU_UPDATE_INTERVAL,
};

/// One row per running process.
pub struct Processes;

impl Processes {
    /// The canonical status names, so that a query written against Linux reads
    /// the same rows on Windows. `sysinfo`'s own `Display` differs per platform
    /// (`Run` vs `Runnable`), and several states only exist on Unix.
    fn status_name(status: ProcessStatus) -> &'static str {
        match status {
            ProcessStatus::Run => "Running",
            ProcessStatus::Sleep => "Sleeping",
            ProcessStatus::Idle => "Idle",
            ProcessStatus::Stop => "Stopped",
            ProcessStatus::Zombie => "Zombie",
            ProcessStatus::Dead => "Dead",
            // Everything the kernel calls "blocked on something": on Unix
            // these are distinct states, on Windows none of them occur.
            ProcessStatus::Tracing
            | ProcessStatus::Waking
            | ProcessStatus::Wakekill
            | ProcessStatus::Parked
            | ProcessStatus::LockBlocked
            | ProcessStatus::UninterruptibleDiskSleep => "Waiting",
            ProcessStatus::Unknown(_) => "Unknown",
        }
    }

    /// A string cell, or NULL when the value is absent or empty. An empty
    /// string would sort and filter as a value; the absence of a command line
    /// we were not allowed to read is not one.
    fn text(value: Option<String>) -> DataValue {
        match value {
            Some(text) if !text.is_empty() => DataValue::String(text),
            _ => DataValue::Null,
        }
    }

    /// Epoch seconds as an ISO timestamp, which is what the date functions
    /// and ORDER BY expect. Zero means "the platform did not say".
    fn timestamp(epoch_seconds: u64) -> DataValue {
        if epoch_seconds == 0 {
            return DataValue::Null;
        }
        match chrono::DateTime::from_timestamp(epoch_seconds as i64, 0) {
            Some(when) => DataValue::DateTime(when.format("%Y-%m-%d %H:%M:%S").to_string()),
            None => DataValue::Null,
        }
    }
}

impl TableGenerator for Processes {
    fn name(&self) -> &str {
        "PROCESSES"
    }

    fn description(&self) -> &str {
        "Every running process (not its threads): pid, parent, owner, status, CPU, memory, start time, command"
    }

    fn arg_count(&self) -> usize {
        0
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn::new("pid").with_type(DataType::Integer),
            DataColumn::new("ppid").with_type(DataType::Integer),
            DataColumn::new("name").with_type(DataType::String),
            DataColumn::new("user").with_type(DataType::String),
            DataColumn::new("status").with_type(DataType::String),
            DataColumn::new("cpu_percent").with_type(DataType::Float),
            DataColumn::new("memory_bytes").with_type(DataType::Integer),
            DataColumn::new("started").with_type(DataType::DateTime),
            DataColumn::new("exe").with_type(DataType::String),
            DataColumn::new("command").with_type(DataType::String),
        ]
    }

    fn generate(&self, _args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        let mut system = System::new();

        // `everything()` rather than the default: the owner and the command
        // line are opt-in refreshes, and without asking for them every row
        // reports NULL for both, which reads as "the OS would not say" rather
        // than "we did not ask".
        let wanted = ProcessRefreshKind::everything();

        // CPU percentage is a rate, so it needs two samples: the first
        // refresh has nothing to compare against and every process reads 0.
        // The wait is `sysinfo`'s own stated minimum and costs ~200ms per
        // call, which is the price of the column being true rather than zero.
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, wanted);
        std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, wanted);

        // Owner names are a separate list: a process carries a user id, not a
        // name. On Windows the id is often unavailable, hence NULL.
        let users = Users::new_with_refreshed_list();

        let mut table = DataTable::new("processes");
        for column in self.columns() {
            table.add_column(column);
        }

        let mut pids: Vec<&Pid> = system.processes().keys().collect();
        pids.sort();

        for pid in pids {
            let process = &system.processes()[pid];

            // Linux lists every *task* - that is, every thread - alongside the
            // processes, each with its own id and the process as its parent.
            // Windows and macOS do not. Including them would break the rule
            // this file exists to keep (the same query returns the same shape
            // everywhere), and it would do real damage to arithmetic: a thread
            // reports its process's memory, so `SUM(memory_bytes)` counts a
            // 4 GB browser once per thread. `thread_kind()` is `None` on every
            // platform but Linux, so this filter costs nothing elsewhere.
            if process.thread_kind().is_some() {
                continue;
            }

            let user = process
                .user_id()
                .and_then(|uid| users.get_user_by_id(uid))
                .map(|user| user.name().to_string());

            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(pid.as_u32() as i64),
                    process
                        .parent()
                        .map_or(DataValue::Null, |p| DataValue::Integer(p.as_u32() as i64)),
                    Self::text(Some(process.name().to_string_lossy().into_owned())),
                    Self::text(user),
                    DataValue::String(Self::status_name(process.status()).to_string()),
                    DataValue::Float(f64::from(process.cpu_usage())),
                    DataValue::Integer(process.memory() as i64),
                    Self::timestamp(process.start_time()),
                    Self::text(
                        process
                            .exe()
                            .map(|path| path.to_string_lossy().into_owned()),
                    ),
                    Self::text(Some(
                        process
                            .cmd()
                            .iter()
                            .map(|arg| arg.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(" "),
                    )),
                ]))
                .map_err(|e| anyhow::anyhow!("processes(): {e}"))?;
        }

        Ok(Arc::new(table))
    }
}

/// One row per socket per owning process (S2).
///
/// This reads the OS's own socket tables - `GetExtendedTcpTable` and
/// `GetExtendedUdpTable` on Windows, netlink `sock_diag` on Linux, `libproc` on
/// macOS - through `netstat2`. **Nothing is resolved to a name.** Reverse DNS
/// is what makes `netstat -a` take seconds; the tables themselves come back in a
/// few milliseconds, and addresses stay as the literal IPs.
pub struct Sockets;

impl Sockets {
    /// RFC 793 spellings, so `WHERE state = 'LISTEN'` means the same thing on
    /// every platform. Ours rather than `netstat2`'s `Display`, which says
    /// `SYN_RCVD` and `__UNKNOWN`.
    fn state_name(state: TcpState) -> &'static str {
        match state {
            TcpState::Listen => "LISTEN",
            TcpState::SynSent => "SYN_SENT",
            TcpState::SynReceived => "SYN_RECEIVED",
            TcpState::Established => "ESTABLISHED",
            TcpState::FinWait1 => "FIN_WAIT_1",
            TcpState::FinWait2 => "FIN_WAIT_2",
            TcpState::CloseWait => "CLOSE_WAIT",
            TcpState::Closing => "CLOSING",
            TcpState::LastAck => "LAST_ACK",
            TcpState::TimeWait => "TIME_WAIT",
            // DELETE_TCB is Windows-only: the connection's control block is
            // being torn down. RFC 793 has no such state; the nearest is the
            // one it is about to reach.
            TcpState::Closed | TcpState::DeleteTcb => "CLOSED",
            TcpState::Unknown => "UNKNOWN",
        }
    }

    fn family(address: &IpAddr) -> &'static str {
        match address {
            IpAddr::V4(_) => "ipv4",
            IpAddr::V6(_) => "ipv6",
        }
    }

    /// The `pid` cells for one socket: one per distinct owner, or a single
    /// NULL when no owner is visible - the socket is real either way, so it
    /// always gets a row.
    ///
    /// Pid 0 is not an owner. Windows reports it for sockets no process holds
    /// any more (a `TIME_WAIT` whose process has exited), and pid 0 is also
    /// the System Idle Process in `processes()` - passing it through would make
    /// the join claim the idle process holds those connections.
    fn owners(pids: &[u32]) -> Vec<DataValue> {
        let mut owners: Vec<u32> = pids.iter().copied().filter(|&pid| pid != 0).collect();
        owners.sort_unstable();
        owners.dedup();
        if owners.is_empty() {
            return vec![DataValue::Null];
        }
        owners
            .into_iter()
            .map(|pid| DataValue::Integer(i64::from(pid)))
            .collect()
    }

    /// Protocol, family, local address and port, remote address and port, and
    /// state - everything but the pid.
    fn socket_cells(socket: &SocketInfo) -> Vec<DataValue> {
        let local_address = socket.local_addr();
        let (protocol, remote, state) = match &socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => {
                // A listener - or a socket not yet connected - has no far end,
                // which the OS reports as 0.0.0.0:0 or [::]:0. That is the same
                // absence as UDP's, and it gets the same NULLs, not an address
                // that would group and filter as if it were one.
                let remote = if tcp.remote_addr.is_unspecified() && tcp.remote_port == 0 {
                    None
                } else {
                    Some((tcp.remote_addr, tcp.remote_port))
                };
                ("tcp", remote, Some(Self::state_name(tcp.state)))
            }
            // UDP is connectionless: no remote end and no state.
            ProtocolSocketInfo::Udp(_) => ("udp", None, None),
        };

        vec![
            DataValue::String(protocol.to_string()),
            DataValue::String(Self::family(&local_address).to_string()),
            DataValue::String(local_address.to_string()),
            DataValue::Integer(i64::from(socket.local_port())),
            remote.map_or(DataValue::Null, |(address, _)| {
                DataValue::String(address.to_string())
            }),
            remote.map_or(DataValue::Null, |(_, port)| {
                DataValue::Integer(i64::from(port))
            }),
            state.map_or(DataValue::Null, |name| DataValue::String(name.to_string())),
        ]
    }

    /// A stable order, so the same machine gives the same result twice: TCP
    /// before UDP, then by local port and address, then by the far end.
    fn sort_key(socket: &SocketInfo) -> (u8, u16, IpAddr, Option<(IpAddr, u16)>) {
        match &socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => (
                0,
                tcp.local_port,
                tcp.local_addr,
                Some((tcp.remote_addr, tcp.remote_port)),
            ),
            ProtocolSocketInfo::Udp(udp) => (1, udp.local_port, udp.local_addr, None),
        }
    }
}

impl TableGenerator for Sockets {
    fn name(&self) -> &str {
        "SOCKETS"
    }

    fn description(&self) -> &str {
        "Every TCP and UDP socket: protocol, local and remote address and port, TCP state, owning pid (joins to processes() on pid)"
    }

    fn arg_count(&self) -> usize {
        0
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn::new("protocol").with_type(DataType::String),
            DataColumn::new("family").with_type(DataType::String),
            DataColumn::new("local_address").with_type(DataType::String),
            DataColumn::new("local_port").with_type(DataType::Integer),
            DataColumn::new("remote_address").with_type(DataType::String),
            DataColumn::new("remote_port").with_type(DataType::Integer),
            DataColumn::new("state").with_type(DataType::String),
            DataColumn::new("pid").with_type(DataType::Integer),
        ]
    }

    fn generate(&self, _args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        let mut sockets = get_sockets_info(
            AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
            ProtocolFlags::TCP | ProtocolFlags::UDP,
        )
        .map_err(|e| anyhow::anyhow!("sockets(): {e}"))?;
        sockets.sort_by_key(Self::sort_key);

        let mut table = DataTable::new("sockets");
        for column in self.columns() {
            table.add_column(column);
        }

        for socket in &sockets {
            let cells = Self::socket_cells(socket);
            // One row per (socket, pid), so `JOIN ... ON pid` stays a plain
            // equi-join. On Linux a listener shared across a fork has several
            // owners; on Windows there is always at most one.
            for pid in Self::owners(&socket.associated_pids) {
                let mut row = cells.clone();
                row.push(pid);
                table
                    .add_row(DataRow::new(row))
                    .map_err(|e| anyhow::anyhow!("sockets(): {e}"))?;
            }
        }

        Ok(Arc::new(table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Linux is the only platform that lists threads alongside processes, so
    /// this is the one assertion that has to be platform-specific. The test
    /// binary runs several threads; before the `thread_kind` filter each of
    /// them appeared as a row naming this process as its parent.
    #[cfg(target_os = "linux")]
    #[test]
    fn threads_are_not_listed_as_processes() {
        let table = Processes.generate(Vec::new()).expect("read processes");
        let me = std::process::id() as i64;

        let claiming_to_be_our_children: Vec<i64> = (0..table.row_count())
            .filter(|&row| table.get_value(row, 1) == Some(&DataValue::Integer(me)))
            .filter_map(|row| match table.get_value(row, 0) {
                Some(DataValue::Integer(pid)) => Some(*pid),
                _ => None,
            })
            .collect();

        assert!(
            claiming_to_be_our_children.is_empty(),
            "the test process spawns no children, so these are its threads: {claiming_to_be_our_children:?}"
        );
    }

    /// Generating is a real call into the OS, so this is one test that does
    /// everything rather than several that each pay the CPU sampling interval.
    #[test]
    fn processes_describes_the_running_machine() {
        let table = Processes.generate(Vec::new()).expect("read processes");

        assert!(
            table.row_count() > 1,
            "a machine running this test has processes"
        );
        assert_eq!(
            table.column_names(),
            vec![
                "pid",
                "ppid",
                "name",
                "user",
                "status",
                "cpu_percent",
                "memory_bytes",
                "started",
                "exe",
                "command",
            ],
            "the column set is fixed, and identical on every platform"
        );

        // The surest fact available: this test is itself a running process.
        let me = std::process::id() as i64;
        let my_row = (0..table.row_count())
            .find(|&row| table.get_value(row, 0) == Some(&DataValue::Integer(me)))
            .expect("the test process should appear in its own process list");

        assert!(
            matches!(table.get_value(my_row, 2), Some(DataValue::String(name)) if !name.is_empty()),
            "a process always has a name"
        );
        assert!(
            matches!(table.get_value(my_row, 6), Some(DataValue::Integer(bytes)) if *bytes > 0),
            "a running process occupies memory"
        );

        // Every row, not just this one: the shape has to hold for processes
        // owned by other users and for ones we may not be allowed to inspect.
        for row in 0..table.row_count() {
            assert!(
                matches!(table.get_value(row, 0), Some(DataValue::Integer(_))),
                "row {row} has no pid"
            );
            assert!(
                matches!(
                    table.get_value(row, 4),
                    Some(DataValue::String(s)) if matches!(
                        s.as_str(),
                        "Running" | "Sleeping" | "Idle" | "Stopped" | "Zombie" | "Dead"
                            | "Waiting" | "Unknown"
                    )
                ),
                "row {row} has a status outside the normalised set: {:?}",
                table.get_value(row, 4)
            );
            // Unreadable cells are NULL, never an empty string masquerading as
            // an answer.
            for column in [3, 8, 9] {
                assert!(
                    !matches!(table.get_value(row, column), Some(DataValue::String(s)) if s.is_empty()),
                    "row {row} column {column} is an empty string; it should be NULL"
                );
            }
        }
    }

    #[test]
    fn a_socket_without_a_visible_owner_still_gets_one_row() {
        assert_eq!(Sockets::owners(&[]), vec![DataValue::Null]);
        // Windows' "no process holds this any more", not the idle process.
        assert_eq!(Sockets::owners(&[0]), vec![DataValue::Null]);
        assert_eq!(
            Sockets::owners(&[42, 7, 42, 0]),
            vec![DataValue::Integer(7), DataValue::Integer(42)],
            "one row per distinct owner, in pid order"
        );
    }

    /// Like `processes()`, one test that does everything, and the surest facts
    /// available are the sockets this test opens itself: a listener, a
    /// connection to it, and a UDP socket, all owned by this process.
    #[test]
    fn sockets_describes_the_running_machine() {
        use std::net::{TcpListener, TcpStream, UdpSocket};

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a listener");
        let listen_port = i64::from(listener.local_addr().unwrap().port());
        let client = TcpStream::connect(listener.local_addr().unwrap()).expect("connect");
        let client_port = i64::from(client.local_addr().unwrap().port());
        let (_server_side, _) = listener.accept().expect("accept");
        let udp = UdpSocket::bind("127.0.0.1:0").expect("bind udp");
        let udp_port = i64::from(udp.local_addr().unwrap().port());

        let table = Sockets.generate(Vec::new()).expect("read sockets");
        assert_eq!(
            table.column_names(),
            vec![
                "protocol",
                "family",
                "local_address",
                "local_port",
                "remote_address",
                "remote_port",
                "state",
                "pid",
            ],
            "the column set is fixed, and identical on every platform"
        );

        let me = DataValue::Integer(std::process::id() as i64);
        let text = |s: &str| DataValue::String(s.to_string());
        let find = |protocol: &str, port: i64, state: Option<&str>| {
            (0..table.row_count())
                .find(|&row| {
                    table.get_value(row, 0) == Some(&text(protocol))
                        && table.get_value(row, 3) == Some(&DataValue::Integer(port))
                        && table.get_value(row, 6) == Some(&state.map_or(DataValue::Null, text))
                })
                .unwrap_or_else(|| panic!("no {protocol} socket on port {port} in state {state:?}"))
        };

        let listening = find("tcp", listen_port, Some("LISTEN"));
        assert_eq!(table.get_value(listening, 1), Some(&text("ipv4")));
        assert_eq!(table.get_value(listening, 2), Some(&text("127.0.0.1")));
        assert_eq!(
            table.get_value(listening, 4),
            Some(&DataValue::Null),
            "a listener has no remote end, so NULL rather than 0.0.0.0"
        );
        assert_eq!(table.get_value(listening, 5), Some(&DataValue::Null));
        assert_eq!(table.get_value(listening, 7), Some(&me), "we own it");

        let connected = find("tcp", client_port, Some("ESTABLISHED"));
        assert_eq!(table.get_value(connected, 4), Some(&text("127.0.0.1")));
        assert_eq!(
            table.get_value(connected, 5),
            Some(&DataValue::Integer(listen_port))
        );
        assert_eq!(table.get_value(connected, 7), Some(&me));

        let datagram = find("udp", udp_port, None);
        assert_eq!(table.get_value(datagram, 4), Some(&DataValue::Null));
        assert_eq!(table.get_value(datagram, 5), Some(&DataValue::Null));
        assert_eq!(table.get_value(datagram, 7), Some(&me));

        // Every row, including sockets owned by other users.
        for row in 0..table.row_count() {
            let protocol = table.get_value(row, 0);
            assert!(
                protocol == Some(&text("tcp")) || protocol == Some(&text("udp")),
                "row {row} has protocol {protocol:?}"
            );
            let family = table.get_value(row, 1);
            assert!(
                family == Some(&text("ipv4")) || family == Some(&text("ipv6")),
                "row {row} has family {family:?}"
            );
            assert!(matches!(
                table.get_value(row, 3),
                Some(DataValue::Integer(_))
            ));
            if protocol == Some(&text("udp")) {
                for column in [4, 5, 6] {
                    assert_eq!(
                        table.get_value(row, column),
                        Some(&DataValue::Null),
                        "row {row}: UDP has no remote end and no state"
                    );
                }
            } else {
                assert!(
                    matches!(
                        table.get_value(row, 6),
                        Some(DataValue::String(s)) if matches!(
                            s.as_str(),
                            "LISTEN" | "SYN_SENT" | "SYN_RECEIVED" | "ESTABLISHED"
                                | "FIN_WAIT_1" | "FIN_WAIT_2" | "CLOSE_WAIT" | "CLOSING"
                                | "LAST_ACK" | "TIME_WAIT" | "CLOSED" | "UNKNOWN"
                        )
                    ),
                    "row {row} has a state outside the normalised set: {:?}",
                    table.get_value(row, 6)
                );
            }
            assert_ne!(
                table.get_value(row, 7),
                Some(&DataValue::Integer(0)),
                "row {row}: pid 0 is not an owner"
            );
        }
    }
}
