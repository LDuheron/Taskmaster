use crate::{Error, Result};
use configparser::ini::Ini;
use std::collections::HashMap;

type ConfigParserContent = HashMap<String, HashMap<String, Option<String>>>;
type RawConfig = HashMap<String, Option<String>>;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum AutorestartOptions {
    Always,
    Never,
    UnexpectedExit,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
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

impl std::cmp::PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
            && self.num_procs == other.num_procs
            && self.auto_start == other.auto_start
            && self.auto_restart == other.auto_restart
            && self.expected_return_codes == other.expected_return_codes
            && self.start_secs == other.start_secs
            && self.start_retries == other.start_retries
            && self.stop_signal == other.stop_signal
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

    fn _parse_raw_config_field<T: std::str::FromStr>(
        raw: &RawConfig,
        field_name: String,
        default: T,
    ) -> Result<T> {
        let type_name: String = std::any::type_name::<T>().into();
        // TODO: try to use unwrap or default
        match raw.get(&field_name) {
            Some(Some(value)) => Ok(value.parse::<T>().map_err(|_| Error::CantParseField {
                field_name,
                value: value.to_string(),
                type_name,
            })?),
            _ => Ok(default),
        }
    }

    fn _parse_autorestart(raw: &RawConfig) -> Result<AutorestartOptions> {
        let field_name = String::from("autorestart");
        match raw.get(&field_name) {
            Some(Some(s)) if *s == String::from("always") => Ok(AutorestartOptions::Always),
            Some(Some(s)) if *s == String::from("never") => Ok(AutorestartOptions::Never),
            Some(Some(s)) if *s == String::from("unexpected") => {
                Ok(AutorestartOptions::UnexpectedExit)
            }
            Some(Some(s)) => Err(Error::FieldBadFormat {
                field_name,
                msg: s.into(),
            }),
            _ => Ok(Job::default().auto_restart),
        }
    }

    fn _parse_autostart(raw: &RawConfig) -> Result<bool> {
        Self::_parse_raw_config_field::<bool>(
            raw,
            String::from("autostart"),
            Job::default().auto_start,
        )
    }

    fn _parse_command(raw: &RawConfig) -> Result<String> {
        let field_name: String = String::from("command");
        match raw.get(&field_name) {
            Some(Some(c)) => {
                let command = c.clone();
                if command.is_empty() {
                    Err(Error::FieldBadFormat {
                        field_name,
                        msg: "Field is empty".into(),
                    })
                } else {
                    Ok(command)
                }
            }
            _ => Err(Error::FieldCommandIsNotSet),
        }
    }

    fn _parse_num_procs(raw: &RawConfig) -> Result<u32> {
        Self::_parse_raw_config_field::<u32>(
            raw,
            String::from("numprocs"),
            Job::default().num_procs,
        )
    }

    fn _parse_job(raw: &RawConfig) -> Result<Job> {
        let command: String = Self::_parse_command(&raw)?;
        let num_procs: u32 = Self::_parse_num_procs(&raw)?;
        let auto_start: bool = Self::_parse_autostart(&raw)?;
        let auto_restart = Self::_parse_autorestart(&raw)?;
        Ok(Job {
            command,
            num_procs,
            auto_start,
            auto_restart,
            ..Default::default()
        })
    }

    pub fn parse_content_of_parserconfig(&mut self, cfg: ConfigParserContent) -> Result<()> {
        for entry in cfg {
            let entry_name = entry.0.clone();
            let job = match Self::_parse_job(&entry.1) {
                Err(e) => {
                    return Err(Error::CantParseEntry {
                        entry_name,
                        e: e.to_string(),
                    })
                }
                Ok(content) => content,
            };
            self.map.insert(entry_name, job);
        }
        self.map.remove("default");
        if self.map.is_empty() {
            return Err(Error::NoJobEntry);
        }
        Ok(())
    }

    pub fn parse_config_file(&mut self, config_path: String) -> Result<()> {
        let mut parser = Ini::new();
        let cfg = parser.load(config_path).unwrap();
        Self::parse_content_of_parserconfig(self, cfg)?;
        println!("{:#?}", self);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Result<T> = std::result::Result<T, Error>;

    fn get_config_parser_and_config(content: String) -> (ConfigParserContent, Config) {
        let config_parser = Ini::new().read(content).unwrap();
        let config = Config::new();
        (config_parser, config)
    }

    #[test]
    fn no_job() -> Result<()> {
        let (config_parser, mut config) = get_config_parser_and_config("".into());
        assert_eq!(
            config.parse_content_of_parserconfig(config_parser),
            Err(Error::NoJobEntry)
        );
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn only_global() -> Result<()> {
        let (config_parser, mut config) = get_config_parser_and_config(String::from(
            "command=test
             numprocs=2",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert_eq!(val, Err(Error::NoJobEntry));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn global_and_another_job() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "command=test
             numprocs=2
             [{job_name}]
             command={command}",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
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
        assert_eq!(config.map.len(), 1);
        Ok(())
    }

    #[test]
    fn default() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command: {command}",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
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
        assert_eq!(config.map.len(), 1);
        Ok(())
    }

    #[test]
    fn empty_command() -> Result<()> {
        let (config_parser, mut config) = get_config_parser_and_config(String::from(
            "[test]
             command:",
        ));
        assert_eq!(
            config.parse_content_of_parserconfig(config_parser),
            Err(Error::CantParseEntry {
                entry_name: String::from("test"),
                e: Error::FieldBadFormat {
                    field_name: "command".into(),
                    msg: "Field is empty".into()
                }
                .to_string(),
            })
        );
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn no_command() -> Result<()> {
        let (config_parser, mut config) = get_config_parser_and_config(String::from(
            "[test]
             numprocs: 2",
        ));
        assert_eq!(
            config.parse_content_of_parserconfig(config_parser),
            Err(Error::CantParseEntry {
                entry_name: String::from("test"),
                e: Error::FieldCommandIsNotSet.to_string(),
            })
        );
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn num_procs_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             numprocs=2",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                num_procs: 2,
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn num_procs_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             numprocs=badnumprocs",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn autostart_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             autostart=false",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                auto_start: false,
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn autostart_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             autostart=badvalue",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn autorestart_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             autorestart=always",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                auto_restart: AutorestartOptions::Always,
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn autorestart_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             autorestart=badvalue",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }
}
