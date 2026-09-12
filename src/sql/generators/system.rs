//! System tables: the running machine as something to query (S1).
//!
//! `SELECT * FROM processes() WHERE name LIKE 'node%'`.
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
}
