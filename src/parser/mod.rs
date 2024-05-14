use crate::{Error, Result};
use configparser::ini::Ini;
use std::collections::HashMap;

type ConfigParserContent = HashMap<String, HashMap<String, Option<String>>>;
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
    command: String,
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
            command: String::new(),
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

// TODO: fix enums
impl std::cmp::PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
            && self.num_procs == other.num_procs
            && self.auto_start == other.auto_start
            // && self.auto_restart == other.auto_restart
            && self.expected_return_codes == other.expected_return_codes
            && self.start_secs == other.start_secs
            && self.start_retries == other.start_retries
            // && self.stop_signal == other.stop_signal
            && self.stop_wait_secs == other.stop_wait_secs
            && self.stdin_file == other.stdin_file
            && self.stdout_file == other.stdout_file
            && self.environment == other.environment
            && self.work_dir == other.work_dir
            && self.umask == other.umask
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

    fn _parse_raw_config_entry<T: std::str::FromStr>(
        raw: &RawConfig,
        entry_name: String,
        default: T,
    ) -> Result<T> {
        let type_name: String = std::any::type_name::<T>().into();
        // TODO: try to use unwrap or default
        match raw.get(&entry_name) {
            Some(Some(b)) => Ok(b.parse::<T>().map_err(|_| Error::CantParseEntry {
                entry_name,
                type_name,
            })?),
            _ => Ok(default),
        }
    }

    fn _parse_autostart(raw: &RawConfig) -> Result<bool> {
        Self::_parse_raw_config_entry::<bool>(
            raw,
            String::from("autostart"),
            Job::default().auto_start,
        )
    }

    fn _parse_command(raw: &RawConfig) -> Result<String> {
        match raw.get("command") {
            Some(Some(c)) => {
                let command = c.clone();
                if command.is_empty() {
                    Err(Error::FieldCommandIsEmpty)
                } else {
                    Ok(command)
                }
            }
            _ => Err(Error::FieldCommandIsNotSet),
        }
    }

    fn _parse_num_procs(raw: &RawConfig) -> Result<u32> {
        Self::_parse_raw_config_entry::<u32>(
            raw,
            String::from("numprocs"),
            Job::default().num_procs,
        )
    }
    fn _parse_job(raw: &RawConfig) -> Result<Job> {
        let command: String = Self::_parse_command(&raw)?;
        let num_procs: u32 = Self::_parse_num_procs(&raw)?;
        let auto_start: bool = Self::_parse_autostart(&raw)?;
        Ok(Job {
            command,
            num_procs,
            auto_start,
            ..Default::default()
        })
    }

    pub fn parse_content_of_parserconfig(&mut self, cfg: ConfigParserContent) -> Result<()> {
        for entry in cfg {
            self.map.insert(entry.0.into(), Self::_parse_job(&entry.1)?);
        }
        Ok(())
    }

    pub fn parse_config_file(&mut self, config_path: String) -> Result<()> {
        let mut parser = Ini::new();
        let cfg = parser
            .load(config_path)
            .map_err(|e| Error::CantLoadFile(e))?;
        Self::parse_content_of_parserconfig(self, cfg)?;
        println!("{:#?}", self);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Error = Box<dyn std::error::Error>;
    type Result<T> = std::result::Result<T, Error>;

    #[test]
    fn default_cfg() -> Result<()> {
        let config_parser = Ini::new().read(String::from(
            "[test]
            command: test",
        ))?;
        let mut config = Config::new();
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get("test").unwrap();
        assert_eq!(
            *job,
            Job {
                command: String::from("test"),
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
            },
        );
        println!("{:#?}", config);
        Ok(())
    }
}
