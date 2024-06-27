use crate::{Error, Result};
use std::fs::OpenOptions;
use std::io::Write;

pub struct Logger {
    log_file: Option<std::fs::File>,
}

impl Default for Logger {
    fn default() -> Self {
        Logger { log_file: None }
    }
}

pub fn log(str: &str) {
    unsafe {
        crate::LOGGER.log_event(str);
    }
}

impl Logger {
    pub const fn new() -> Self {
        Logger { log_file: None }
    }

    pub fn init(self: &mut Self, file_name: &str) -> Result<()> {
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_name)
        {
            Ok(file) => {
                self.log_file = Some(file);
                Ok(())
            }
            Err(e) => Err(Error::CantOpenLogFile(e.to_string())),
        }
    }

    pub fn log_event(self: &mut Self, text: &str) {
        if let Some(ref mut file) = self.log_file {
            let to_log = String::from(text) + "\n";
            let data: &[u8] = to_log.as_bytes();
            let _ = file.write(data);
        } else {
            eprintln!("Error: No log file");
            return;
        }
    }
}
