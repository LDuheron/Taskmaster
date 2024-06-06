mod error;
mod parser;

use error::{Error, Result};
use parser::config::Config;
use std::io::prelude::*;

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

fn try_reload_config(config: &mut Config, config_file: &String) {
    unsafe {
        if RELOAD_CONFIG {
            match config.reload_config(&config_file) {
                Err(e) => println!("log: can't reload file: {e}"),
                Ok(()) => println!("config file is reloaded: \n{:#?}", config),
            }
            RELOAD_CONFIG = false;
        }
    }
}

fn init_connection(ip: String, port: String) -> Result<Option<std::net::TcpListener>> {
    let listener = std::net::TcpListener::bind(format!("{ip}:{port}"))
        .map_err(|err| Error::Default(err.to_string()))?;
    if listener.set_nonblocking(true).is_err() {
        println!("Can't set non blocking listener...");
        return Ok(None);
    }
    println!("bind ok");
    Ok(Some(listener))
}

fn main() -> Result<()> {
    if std::env::args().len() != 2 {
        return Err(Error::BadNumberOfArguments(String::from(
            "usage: taskmaster config_file",
        )));
    }
    let config_file: String = std::env::args().nth(1).unwrap();
    let mut config: Config = Config::new();
    config.parse_config_file(&config_file)?;
    println!("{:#?}", config);
    unsafe {
        signal(SIGHUP, handle_sighup as usize);
    }
    let duration = std::time::Duration::from_millis(100);
    let listener = match init_connection("127.0.0.1".into(), "4241".into())? {
        Some(l) => l,
        None => todo!(),
    };
    for stream in listener.incoming() {
        try_reload_config(&mut config, &config_file);
        match stream {
            Ok(mut s) => {
                let mut str = String::new();
                match s.read_to_string(&mut str) {
                    Ok(bytes) if bytes != 0 => {
                        println!("read: {}", str);
                    }
                    Ok(bytes) if bytes == 0 => {
                        println!("client disconnected");
                    }
                    _ => std::thread::sleep(duration),
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(duration)
            }
            Err(e) => return Err(Error::Default(format!("encountered IO error: {e}"))),
        }
    }
    Ok(())
}
