use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use std::sync::OnceLock;

pub static DEBUG_FILE: OnceLock<Mutex<Option<std::fs::File>>> = OnceLock::new();

pub fn init_debug_log() {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("tui_debug.log")
        .ok();

    let _ = DEBUG_FILE.set(Mutex::new(file));
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            if let Some(debug_file) = $crate::utils::debug_helpers::DEBUG_FILE.get() {
                if let Ok(mut guard) = debug_file.lock() {
                    if let Some(ref mut file) = *guard {
                        let _ = writeln!(file, "[{}] {}",
                            chrono::Local::now().format("%H:%M:%S%.3f"),
                            format!($($arg)*));
                        let _ = file.flush();
                    }
                }
            }
        }
    };
}

pub fn debug_breakpoint(label: &str) {
    #[cfg(debug_assertions)]
    {
        debug_log!("BREAKPOINT: {}", label);

        // This allows you to set a breakpoint here in RustRover
        // The label will be logged so you know which point was hit
        let _debug_marker = format!("Debug point: {}", label);
    }
}
