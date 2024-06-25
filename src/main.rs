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

fn parse_cmd_from_client_input(raw: &String) -> Result<String> {
    if let Some(cmd) = raw.split_whitespace().next() {
        Ok(cmd.to_string())
    } else {
        Err(Error::WrongClientInputFormat)
    }
}

fn parse_arg_from_client_input(raw: &String) -> Result<String> {
    if let Some(cmd) = raw.split_whitespace().skip(1).next() {
        let index = cmd.rfind(":");
        if index.is_some() {
            let split_cmd = &cmd[0..index.unwrap()];
            println!("{:?}", split_cmd.to_string());
            Ok(split_cmd.to_string())
        } else {
            println!("{:?}", cmd.to_string());
            Ok(cmd.to_string())
        }
    } else {
        Err(Error::WrongClientInputFormat) // repondre au client
    }
}

fn parse_target_process_number_from_client_input(
    raw: &String
) -> Result<Option<u32>> {
    if let Some(cmd) = raw.split_whitespace().skip(1).next() {
        if let Some(index) = cmd.rfind(":") {
            let split_cmd = &cmd[index + 1..];
            if let Ok(number) = split_cmd.parse::<u32>() {
                    return Ok(Some(number));        
            }
			else {
				return Err(Error::WrongClientInputFormat);
			}
        }
    }
    Ok(None)
}

fn is_job_from_config_map(config: &mut Config, cmd: &String) -> bool {
    let result = config.contains_key(cmd);
    if result == true {
        return true;
    }
    return false;
}

fn parse_client_input(config: &mut Config, raw: &String) -> Result<(String, String, Option<u32>)> {
    let client_cmd = parse_cmd_from_client_input(&raw)?;
    let client_arg = parse_arg_from_client_input(&raw)?;
    let client_process = parse_target_process_number_from_client_input(&raw)?;
    if is_job_from_config_map(config, &client_arg) {
        Ok((client_cmd, client_arg, client_process))
    } else {
        Err(Error::WrongClientInputFormat)
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
                let formatted = String::from_utf8_lossy(&data[..bytes_read]).into_owned();
                println!("read: {}", formatted);
                // do something with the message from the client
                // and return a message
                // is it a fatal error ?
                let (client_cmd, client_arg, client_process) =
                    if let Ok((client_cmd, client_arg, client_process)) =
                        parse_client_input(config, &formatted)
                    {
                        (client_cmd, client_arg, client_process)
                    } else {
                        s.write(b"Error while parsing the input!")
                            .map_err(|e| Error::IO(e.to_string()))?;
                        continue;
                    };
                let job: &mut parser::job::Job = config.get_mut(&client_arg).unwrap();
                match client_cmd.as_str() {
                    "start" => {
                        job.start(&client_arg, client_process);
                    }
                    "stop" => {
                        job.stop(&client_arg, client_process);
                    }
                    "restart" => {
                        job.restart(&client_arg, client_process);
                    }
                    _ => {
                        s.write(b"Unknown command: Please try start, stop or restart")
                            .map_err(|e| Error::IO(e.to_string()))?;
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
