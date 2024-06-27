mod config;
mod error;
mod job;
mod logger;
mod parsing;

use config::Config;
use error::{Error, Result};
use job::Job;
use logger::{log, Logger};
use parsing::{
    parse_arg_from_client_input, parse_cmd_from_client_input,
    parse_target_process_number_from_client_input,
};
use std::env::args;
use std::io::{prelude::*, ErrorKind};
use std::net::TcpListener;
use std::thread::sleep;
use std::time::Duration;

const SIGHUP: i32 = 1;
static mut RELOAD_CONFIG: bool = false;
static mut LOGGER: Logger = Logger::new();

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
                Err(e) => {
                    log(&format!("ERROR: Can't reload file: {e}"));
                }
                Ok(()) => {
                    log(&format!("INFO: Config file is reloaded"));
                }
            }
            RELOAD_CONFIG = false;
        }
    }
}

fn is_job_from_config_map(config: &mut Config, cmd: &String) -> bool {
    let result = config.contains_key(cmd);
    if result == true {
        return true;
    }
    return false;
}

fn parse_client_input(
    config: &mut Config,
    raw: &String,
) -> Result<(String, String, Option<usize>)> {
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
    let duration = Duration::from_millis(100);
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
                    sleep(duration);
                    continue;
                }
                let formatted = String::from_utf8_lossy(&data[..bytes_read]).into_owned();
                println!("read: {}", formatted);
                if formatted.trim() == "status".to_string() {
                    s.write(&config.status().into_bytes())
                        .map_err(|e| Error::IO(e.to_string()))?;
                    continue;
                }
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
                let job: &mut Job = config.get_mut(&client_arg).unwrap();
                let ret = match client_cmd.as_str() {
                    "start" => job.start(&client_arg, client_process),
                    "stop" => job.stop(&client_arg, client_process),
                    "restart" => job.restart(&client_arg, client_process),
                    "status" => job.status(&client_arg, client_process),
                    _ => Err(Error::CommandIsNotSuported(
                        "Unknown command: Please try start, stop or restart!".into(),
                    )),
                };
                if ret.is_err() {
                    s.write(&ret.unwrap_err().to_string().into_bytes())
                        .map_err(|e| Error::IO(e.to_string()))?;
                } else {
                    s.write(&ret.unwrap().into_bytes())
                        .map_err(|e| Error::IO(e.to_string()))?;
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => sleep(duration),
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
    Ok(listener)
}

fn main() -> Result<()> {
    if args().len() != 2 {
        return Err(Error::BadNumberOfArguments(String::from(
            "usage: taskmaster config_file",
        )));
    }
    unsafe {
        LOGGER.init("taskmaster.log")?;
    }
    let config_file: String = args().nth(1).unwrap();
    let mut config: Config = Config::new();
    config.parse_config_file(&config_file)?;
    config.run_autostart_jobs();
    println!("{:#?}", config);
    unsafe {
        signal(SIGHUP, handle_sighup as usize);
    }
    let listener: TcpListener = init_connection("localhost".into(), "4241".into())?;
    server_routine(&listener, &mut config, &config_file)?;
    Ok(())
}
