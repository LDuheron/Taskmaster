use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, Once};

static START: Once = Once::new();
static mut LOGGER: Option<Mutex<Logger>> = None;

pub struct Logger {
    pub log_file: Option<std::fs::File>,
}

impl Default for Logger {
    fn default() -> Self {
        Logger { log_file: None }
    }
}

impl Logger {
    pub fn init_log() {
        START.call_once(|| {
            let logger = Logger::default();

            match OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open("log.txt")
            {
                Ok(file) => {
                    logger.log_file = Some(file);
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    return;
                }
            }

            unsafe {
                Logger = Some(Mutex::new(logger));
            }
        });
    }

    pub fn log_event(self: &mut Self, text: &String) {
        if let Some(ref mut file) = self.log_file {
            let data: &[u8] = text.as_bytes();
            let _ = file.write(data);
        } else {
            eprintln!("Error: No log file");
            return;
            //    Err(Error::NoLogFile)
        }
    }
}
