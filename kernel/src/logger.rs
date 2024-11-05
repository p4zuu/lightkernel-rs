use super::io::IOPort;
use core::fmt;
use log::SetLoggerError;
use spin::Lazy;

static mut CONSOLE: IOPort = IOPort::new(0x03F8);
static LOGGER: Lazy<Logger> = Lazy::new(Logger::new);

#[derive(Default)]
struct Logger;

impl Logger {
    const fn new() -> Self {
        Self
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match record.metadata().level() {
            log::Level::Warn | log::Level::Debug | log::Level::Trace => {
                unimplemented!();
            }
            log::Level::Error => _log(format_args!("[ERROR] {}", record.args())),
            log::Level::Info => _log(format_args!("[KERNEL] {}", record.args())),
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&*LOGGER)?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}

pub fn _log(args: fmt::Arguments<'_>) {
    use core::fmt::Write;
    // SAFETY: mutliple CPU are unsupported
    unsafe {
        #[allow(static_mut_refs)]
        write!(CONSOLE, "{args}\r\n").unwrap();
    }
}
