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

///// Start parsing client input

fn _parse_client_cmd(raw: &String) -> Result<String> {
    if let Some(cmd) = raw.split_whitespace().next() {
        Ok(cmd.to_string())
    } else {
        Err(Error::FieldCommandIsNotSet)
    }
}

fn _parse_client_arg(raw: &String) -> Result<String> {
    if let Some(cmd) = raw.split_whitespace().skip(1).next() {
        let index = cmd.find(":");
        if index.is_some() {
            let split_cmd = &cmd[0..index.unwrap()];
            println!("{:?}", split_cmd.to_string());
            Ok(split_cmd.to_string())
        } else {
            println!("{:?}", cmd.to_string());
            Ok(cmd.to_string())
        }
    } else {
        Err(Error::FieldCommandIsNotSet) // repondre au client + new error
    }
}

fn _parse_client_process(raw: &String, client_arg: &String) -> Result<Option<u32>> {
    if let Some(cmd) = raw.split_whitespace().skip(1).next() {
        if let Some(index) = cmd.find(":") {
            let split_cmd = &cmd[index + 1..];
            if let Some(last_split_index) = split_cmd.rfind("_") {
                let (arg, num_proc_str) = split_cmd.split_at(last_split_index);
				let num_proc_str = &num_proc_str[1..];
                if arg == *client_arg {
                    if let Ok(number) = num_proc_str.parse::<u32>() {
                        return Ok(Some(number));
                    }
                }
            }
        }
    }
    Ok(None)
}

fn is_job(config: &mut Config, cmd: &str) -> bool {
    let keys = config.get_all_keys();
    for key in keys {
        println!("{}", key);
        if key == cmd {
            return true;
        }
    }
    return false;
}

fn parse_client_input(config: &mut Config, raw: &String) -> Result<(String, String, Option<u32>)> {
    let client_cmd = _parse_client_cmd(&raw)?;
    let client_arg = _parse_client_arg(&raw)?;
    let client_process = _parse_client_process(&raw, &client_arg)?;
    if is_job(config, &client_arg) {
        Ok((client_cmd, client_arg, client_process))
    } else {
        Err(Error::FieldCommandIsNotSet) ///// Mettre une erreur appropriee
    }
}

///// End parsing client input

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
                let formatted = String::from_utf8_lossy(&data[..bytes_read]).into_owned();
                println!("read: {}", formatted);
                // do something with the message from the client
                // and return a message
                // is it a fatal error ?
                match parse_client_input(config, &formatted) {
                    Ok((client_cmd, client_arg, client_process)) => {
                        println!("client cmd : {:?}", client_cmd); //
                        println!("client arg : {:?}", client_arg); //
                        println!("client process : {:?}\n", client_process); //

                        // pass the target process as parameter of the start stop restart functions
                        match client_cmd.as_str() {
                            "start" => {
                                config
                                    .get_mut(&client_arg)
                                    .unwrap()
                                    .start(&client_arg, client_process);
                            }
                            "stop" => {
                                config.get_mut(&client_arg).unwrap().stop(&client_arg);
                            }
                            "restart" => {
                                config
                                    .get_mut(&client_arg)
                                    .unwrap()
                                    .restart(&client_arg, client_process);
                            }
                            _ => eprintln!("Unknown command: Please try start, stop or restart"),
                        }
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                        continue;
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
