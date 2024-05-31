mod error;
mod parser;

// https://doc.rust-lang.org/std/net/struct.TcpListener.html
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html

// Erreurs : si le server exit, le client continue d'envoyer des messages
// Gerer le controle c cote serveur
// gerer plusieurs clients -> bloquer les autres

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use error::{Error, Result};
use parser::config::Config;

fn handle_connection(mut stream: TcpStream) -> Result<()> {
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

fn init_server() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4243").unwrap_or_else(|err| {
        panic!("Failed to bind");
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Info : New connection");
                let _ = handle_connection(stream);
            }
            Err(_) => {
                println!("Error during connection");
            }
        }
        {}
        println!("Info : Client disconnected");
    }

    Ok(())
}

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
    unsafe {
        // TODO: change to sighup
        signal(3, handle_sighup as usize);
    }
    if std::env::args().len() != 2 {
        return Err(Error::BadNumberOfArguments(String::from(
            "usage: taskmaster config_file",
        )));
    }
    let config_file: String = std::env::args().nth(1).unwrap();
    // let mut config: Config = Config::new();
    // config.parse_config_file(&config_file)?;
    println!("{:#?}", config);
    loop {
        unsafe {
            if RELOAD_CONFIG {
                config.reload_config(&config_file)?;
    init_server()?;
                RELOAD_CONFIG = false;
            }
        }
        let duration = std::time::Duration::from_millis(500);
        std::thread::sleep(duration);
    }
    // Ok(())
}
