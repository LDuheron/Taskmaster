use std::fs::OpenOptions;
use std::io::Write;
pub struct Logger {
    pub log_file: Option<std::fs::File>,
}

impl Default for Logger {
    fn default() -> Self {
        Logger { log_file: None }
    }
}

impl Logger {
    pub fn new(self: &mut Self) {
        match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("log.txt")
        {
            Ok(file) => {
                self.log_file = Some(file);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return;
            }
        }
    }

    pub fn log_event(self: &mut Self, text: &String) {
        if let Some(ref mut file) = self.log_file {
			let data: &[u8] = text.as_bytes();	
            let _ = file.write(data);
        }
		else {
			eprintln!("Error: No log file");
			return;
		}
    }
}
