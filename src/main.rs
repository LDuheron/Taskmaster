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

fn init_server() -> Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:4241")
        .map_err(|err| Error::Default(err.to_string()))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection");
                handle_connection(stream)?;
            }
            Err(_) => {
                println!("Error during connection");
            }
        }
        {}
        println!("Debug : has skipped handle connection");
    }

    Ok(())
}

fn handle_connection(mut stream: std::net::TcpStream) -> Result<()> {
    println!("In handle connection");
    loop {
        let mut data = [0; 512];
        let bytes_read = stream
            .read(&mut data)
            .map_err(|e| Error::Default(e.to_string()))?;
        if bytes_read == 0 {
            println!("Client is disconnected");
            break;
        }
        let formatted = String::from_utf8_lossy(&data[..bytes_read]);
        println!("Received: {}", formatted);
    }
    Ok(())
}

fn main() -> Result<()> {
    unsafe {
        signal(SIGHUP, handle_sighup as usize);
    }
    if std::env::args().len() != 2 {
        return Err(Error::BadNumberOfArguments(String::from(
            "usage: taskmaster config_file",
        )));
    }
    let config_file: String = std::env::args().nth(1).unwrap();
    let mut config: Config = Config::new();
    config.parse_config_file(&config_file)?;
    println!("{:#?}", config);
    init_server()?;
    let duration = std::time::Duration::from_millis(500);
    loop {
        unsafe {
            if RELOAD_CONFIG {
                config.reload_config(&config_file)?;
                println!("{:#?}", config);
                RELOAD_CONFIG = false;
            }
        }
        std::thread::sleep(duration);
    }
    // Ok(())
}
// https://doc.rust-lang.org/std/net/struct.TcpListener.html
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html
