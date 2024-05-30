mod error;
mod parser;

use error::{Error, Result};
use parser::config::Config;

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
    unsafe {
        signal(1, handle_sighup as usize);
    }
    let mut config: Config = Config::new();
    config.parse_config_file(String::from("config.ini"))?;
    println!("{:#?}", config);
    loop {
        unsafe {
            if RELOAD_CONFIG {
                println!("RELOAD_CONFIG");
                RELOAD_CONFIG = false;
            }
        }
    }
    Ok(())
}
