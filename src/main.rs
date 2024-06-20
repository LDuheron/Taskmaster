mod error;
mod parser;

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

fn _parse_client_cmd(raw: &String) -> Result<String> {
    if let Some(cmd) = raw.split_whitespace().next() {
        let index = cmd.find(":");
        if index.is_some() {
            // TODO!          // let mut split_cmd =

            Ok(cmd.to_string())
        } else {
            Ok(cmd.to_string())
        }
    } else {
        Err(Error::FieldCommandIsNotSet)
    }
}

fn _parse_client_arg(raw: &String) -> Result<String> {
    if let Some(cmd) = raw.split_whitespace().skip(1).next() {
        Ok(cmd.to_string())
    } else {
        Err(Error::FieldCommandIsNotSet) // repondre au client + new errror
    }
}

fn server_routine(listener: &TcpListener, config: &mut Config, config_file: &String) -> Result<()> {
    let duration = std::time::Duration::from_millis(100);
    for stream in listener.incoming() {
        try_reload_config(config, config_file);
        config.jobs_routine();
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
                let formatted = String::from_utf8_lossy(&data[..bytes_read]).into_owned();
                println!("read: {}", formatted);
                let client_cmd = _parse_client_cmd(&formatted);
                let client_arg = _parse_client_arg(&formatted);
                // TODO
                // let client_target_process: _parse_client_process(formatted);
                match client_cmd {
                    Ok(cmd) if cmd == "start" => {
                        config
                            .get_mut(&String::from("term"))
                            .unwrap()
                            .start(&String::from("term")); // error
                    }
                    Ok(cmd) if cmd == "stop" => {
                        println!("stop");
                        if client_arg.is_ok() {
                            config
                                .get_mut(&String::from("term"))
                                .unwrap()
                                .stop(&String::from("term")); // error
                        }
                    }
                    Ok(cmd) if cmd == "restart" => {
                        println!("restart");
                    }
                    Ok(_) => todo!(),
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }

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

// to do -> parser le job
// split si num proc >
