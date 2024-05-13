use crate::{Error, Result};
use configparser::ini::Ini;
use std::collections::HashMap;

type RawConfig = HashMap<String, Option<String>>;

#[derive(Debug)]
pub enum AutorestartOptions {
    Always,
    Never,
    UnexpectedExit,
}

#[derive(Debug)]
pub enum StopSignals {
    SIGHUP = 1,
    SIGINT = 2,
    SIGQUIT = 3,
    SIGKILL = 9,
    SIGUSR1 = 10,
    SIGUSR2 = 12,
    SIGTERM = 15,
}

#[derive(Debug)]
pub struct Job {
    command: Option<String>,
    num_procs: u32,
    auto_start: bool,
    auto_restart: AutorestartOptions,
    expected_return_codes: Vec<u32>,
    start_secs: u32,
    start_retries: u32,
    stop_signal: Vec<StopSignals>,
    stop_wait_secs: u32,
    stdin_file: Option<String>,
    stdout_file: Option<String>,
    environment: HashMap<String, String>,
    work_dir: Option<String>,
    umask: Option<String>,
}

impl Default for Job {
    fn default() -> Self {
        Job {
            command: None,
            num_procs: 1,
            auto_start: true,
            auto_restart: AutorestartOptions::UnexpectedExit,
            expected_return_codes: vec![1],
            start_secs: 1,
            start_retries: 3,
            stop_signal: vec![StopSignals::SIGTERM],
            stop_wait_secs: 10,
            stdin_file: None,
            stdout_file: None,
            environment: HashMap::new(),
            work_dir: None,
            umask: None,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub map: HashMap<String, Job>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            map: HashMap::new(),
        }
    }

    fn _parse_command(raw: &RawConfig) -> Result<Option<String>> {
        match raw.get("command") {
            Some(c) => {
                let command = c.clone();
                if command.as_ref().unwrap().is_empty() {
                    Err(Error::FieldCommandIsEmpty)
                } else {
                    Ok(command)
                }
            }
            None => Err(Error::FieldCommandIsNotSet),
        }
    }

    fn _parse_num_procs(raw: &RawConfig) -> Result<u32> {
        match raw.get("numprocs") {
            Some(Some(s)) => match s.parse::<u32>() {
                Ok(n) => Ok(n),
                Err(_) => Err(Error::FieldNumProcsIsNotPositiveNumber { str: s.into() }),
            },
            _ => Ok(Job::default().num_procs),
        }
    }

    fn _parse_job(raw: &RawConfig) -> Result<Job> {
        let command: Option<String> = Self::_parse_command(&raw)?;
        let num_procs: u32 = Self::_parse_num_procs(&raw)?;
        Ok(Job {
            command,
            num_procs,
            ..Default::default()
        })
    }

    pub fn parse_config_file(&mut self, config_path: String) -> Result<()> {
        let mut parser = Ini::new();
        let cfg = parser.load(config_path)?;
        // without Error::Default we can do this:
        // .map_err(|e| Error::SomeError(e))?;
        for entry in &cfg {
            self.map.insert(entry.0.into(), Self::_parse_job(&entry.1)?);
        }
        println!("{:#?}", self);
        Ok(())
    }
}
