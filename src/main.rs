mod error;
mod parser;

use error::{Error, Result};
use parser::config::Config;

const SIGHUP: i32 = 1;
static mut RELOAD_CONFIG: bool = false;

extern "C" {
    pub fn signal(signum: i32, handler: usize) -> u32;
}

extern "C" fn handle_sighup(_signum: i32) {
    unsafe {
        RELOAD_CONFIG = true;
    }
}

fn main() -> Result<()> {
    if std::env::args().len() != 2 {
        return Err(Error::BadNumberOfArguments(String::from(
            "usage: taskmaster config_file",
        )));
    }
    let config_file: String = std::env::args().nth(1).unwrap();
    unsafe {
        signal(SIGHUP, handle_sighup as usize);
    }
    let mut config: Config = Config::new();
    config.parse_config_file(config_file)?;
    println!("{:#?}", config);
    loop {
        unsafe {
            if RELOAD_CONFIG {
                println!("RELOAD_CONFIG");
                // config.reload_config(config_file);
                RELOAD_CONFIG = false;
            }
        }
        let duration = std::time::Duration::from_millis(500);
        std::thread::sleep(duration);
    }
    // Ok(())
}
