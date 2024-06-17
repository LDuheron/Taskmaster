mod error;
mod parser;

// https://doc.rust-lang.org/std/net/struct.TcpListener.html
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html

// Erreurs : si le server exit, le client continue d'envoyer des messages
// Gerer le controle c cote serveur
// gerer plusieurs clients -> bloquer les autres
use error::{Error, Result};
use parser::config::Config;
use std::io::prelude::*;
use std::net::TcpListener;

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
                Ok(()) => println!("log: config file is reloaded: \n{:#?}", config),
            }
            RELOAD_CONFIG = false;
        }
    }
}

fn server_routine(listener: &TcpListener, config: &mut Config, config_file: &String) -> Result<()> {
    let duration = std::time::Duration::from_millis(100);
    for stream in listener.incoming() {
        try_reload_config(config, config_file);
        match stream {
            Ok(mut s) => {
                let mut data: [u8; 128] = [0; 128];
                let bytes_read: usize = s
                    .read(&mut data)
                    .map_err(|e| Error::Default(e.to_string()))?;
                if bytes_read == 0 {
                    std::thread::sleep(duration);
                    continue;
                }
                let formatted = String::from_utf8_lossy(&data[..bytes_read]);
                println!("read: {}", formatted);
                // do something with the message from the client
                // and return a message
                // is it a fatal error ?
                s.write(b"Success").map_err(|e| Error::IO(e.to_string()))?;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(duration)
            }
            Err(e) => return Err(Error::IO(e.to_string())),
        }
    }
    Ok(())
}

fn init_connection(ip: String, port: String) -> Result<TcpListener> {
    let listener =
        TcpListener::bind(format!("{ip}:{port}")).map_err(|err| Error::Default(err.to_string()))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| Error::Default(err.to_string()))?;
    println!("bind ok");
    Ok(listener)
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
    let listener: TcpListener = init_connection("localhost".into(), "4241".into())?;
    server_routine(&listener, &mut config, &config_file)?;
    Ok(())
}
