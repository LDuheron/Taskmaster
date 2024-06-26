use super::job::{AutorestartOptions, Job, StopSignals};
use crate::{Error, Result};
use configparser::ini::Ini;
use std::collections::HashMap;

pub type ConfigParserContent = HashMap<String, HashMap<String, Option<String>>>;
pub type RawConfig = HashMap<String, Option<String>>;

#[derive(Debug, Clone)]
pub struct Config {
    map: HashMap<String, Job>,
}

impl Config {
    //
    //
    // -- PUBLIC
    //

    pub fn new() -> Self {
        Config {
            map: HashMap::new(),
        }
    }

    pub fn get_mut(&mut self, key: &String) -> Option<&mut Job> {
        self.map.get_mut(key)
    }

    pub fn contains_key(&mut self, key: &String) -> bool {
        self.map.contains_key(key)
    }

    pub fn reload_config(&mut self, config_path: &String) -> Result<()> {
        let mut old_config: Config = self.clone();
        self.map.clear();
        let parsing_result = Self::parse_config_file(self, config_path);
        if parsing_result.is_err() {
            println!(
                "log: cant reload the config: {:?}",
                parsing_result.unwrap_err()
            );
            self.map = old_config.map;
            return Ok(());
        }
        println!("log: reload config with {}", config_path);
        for entry in self.map.iter_mut() {
            let job_name: String = entry.0.into();
            let job: &mut Job = entry.1;
            let old_job: &mut Job = match old_config.map.get_mut(&job_name) {
                Some(j) => j,
                // new job case
                _ => {
                    if job.auto_start {
                        job.start(&job_name, None); // TODO ! set None as target process for compilation error
                    }
                    continue;
                }
            };
            // job is changed case
            if job != old_job {
                // TODO: check if the job is running and handle this
                old_job.start(&job_name, None); // TODO ! set None as target process for compilation error
                                                //     old_job.stop(&job_name);
                                                //     job.start(&job_name);
                                                // } else if job.auto_start {
                                                //     job.start(&job_name);
                                                // }
            }
            old_config.map.remove_entry(&job_name);
        }
        // job is not present in new config file
        for entry in old_config.map.iter_mut() {
            let old_job_name: String = entry.0.into();
            let old_job: &mut Job = entry.1;
            old_job.stop(&old_job_name, None); // TODO ! set None as target process for compilation error
        }
        Ok(())
    }

    pub fn parse_content_of_parserconfig(&mut self, cfg: ConfigParserContent) -> Result<()> {
        for entry in cfg {
            let entry_name: String = entry.0.clone();
            let job: Job = match Self::_parse_job(&entry.1) {
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

    pub fn parse_config_file(&mut self, config_path: &String) -> Result<()> {
        let mut parser: Ini = Ini::new();
        let cfg: ConfigParserContent = parser
            .load(config_path)
            .map_err(|e| Error::CantLoadFile(e.to_string()))?;
        self.parse_content_of_parserconfig(cfg)?;
        for entry in self.map.iter_mut() {
            let job_name: &String = entry.0;
            let job: &mut Job = entry.1;
            if job.auto_start {
                job.start(job_name, None); // TODO ! set None as target process for compilation error
            }
        }
        Ok(())
    }

    //
    // -- PRIVATE
    //

    fn _parse_raw_config_field<T: std::str::FromStr>(
        raw: &RawConfig,
        field_name: String,
        default: T,
    ) -> Result<T> {
        match raw.get(&field_name) {
            Some(Some(value)) => Ok(value.parse::<T>().map_err(|_| Error::CantParseField {
                field_name,
                value: value.to_string(),
                type_name: std::any::type_name::<T>().into(),
            })?),
            _ => Ok(default),
        }
    }

    fn _parse_one_word_field(
        raw: &RawConfig,
        field_name: String,
        default: Option<String>,
    ) -> Result<Option<String>> {
        match raw.get(&field_name) {
            Some(Some(str)) => {
                if str.contains(char::is_whitespace) {
                    Err(Error::FieldBadFormat {
                        field_name,
                        msg: "Field contain space".into(),
                    })
                } else {
                    if str.is_empty() {
                        Ok(default)
                    } else {
                        Ok(Some(str.clone()))
                    }
                }
            }
            _ => Ok(default),
        }
    }

    fn _parse_command_line(raw: &RawConfig) -> Result<String> {
        let file_name: Option<String> = Some(Self::_parse_raw_config_field(
            raw,
            "command".into(),
            String::new(),
        )?);
        if let Some(cmd_as_str) = file_name {
            Ok(cmd_as_str)
        } else {
            Err(Error::FieldCommandIsNotSet)
        }
    }

    fn _parse_arguments(raw: &RawConfig) -> Result<Option<Vec<String>>> {
        let cmd = Some(Self::_parse_command_line(raw)?);
        if let Some(cmd_as_str) = cmd {
            let args: Vec<String> = cmd_as_str
                .split_whitespace()
                .skip(1)
                .map(String::from)
                .collect();
            if args.is_empty() {
                Ok(None)
            } else {
                Ok(Some(args))
            }
        } else {
            Err(Error::FieldCommandIsNotSet)
        }
    }

    fn _parse_command(raw: &RawConfig) -> Result<String> {
        let string = Some(Self::_parse_command_line(raw)?);
        if let Some(cmd_as_str) = string {
            if let Some(cmd) = cmd_as_str.split_whitespace().next() {
                Ok(cmd.to_string())
            } else {
                Err(Error::FieldCommandIsNotSet)
            }
        } else {
            Err(Error::FieldCommandIsNotSet)
        }
    }

    fn _parse_umask(raw: &RawConfig) -> Result<Option<u32>> {
        let field_name: String = String::from("umask");
        let default: Option<String> = None;
        let Some(umask_str) = Self::_parse_one_word_field(&raw, field_name.clone(), default)?
        else {
            return Ok(None);
        };
        let is_valid_umask: bool =
            umask_str.len() == 3 && umask_str.chars().all(|c| matches!(c, '0'..='8'));
        if is_valid_umask {
            let umask = umask_str.parse::<u32>();
            match umask {
                Ok(umask) => Ok(Some(umask)),
                Err(_e) => Err(Error::FieldBadFormat {
                    field_name,
                    msg: "Field contain too much characters".into(),
                }), // passer de base octale a la base decimale
            }
        } else {
            Err(Error::FieldBadFormat {
                field_name,
                msg: "Field contain too much characters".into(),
            })
        }
    }

    fn _parse_environment(raw: &RawConfig) -> Result<Option<HashMap<String, String>>> {
        let field_name: String = String::from("environment");
        let default: Option<HashMap<String, String>> = Job::default().environment;
        let Some(Some(raw_env)) = raw.get(&field_name) else {
            return Ok(default);
        };
        let env_entry: Vec<&str> = raw_env.split(",").collect();
        let mut map: HashMap<String, String> = HashMap::new();
        for entry in env_entry {
            let Some(pos_first_equal) = entry.find("=") else {
                return Err(Error::CantParseEnvEntry(entry.to_string()));
            };
            let key = entry[..pos_first_equal].to_string();
            let mut value: &str = &entry[pos_first_equal + 1..];
            if value.starts_with('"') && value.ends_with('"') {
                value = &value[1..value.len() - 1];
            }
            map.insert(key.to_string(), value.to_string());
        }
        Ok(Some(map))
    }

    fn _parse_working_directory(raw: &RawConfig) -> Result<Option<String>> {
        Ok(Self::_parse_one_word_field(
            &raw,
            "workdir".into(),
            Job::default().stderr_file,
        )?)
    }

    fn _parse_stderr_file(raw: &RawConfig) -> Result<Option<String>> {
        Ok(Self::_parse_one_word_field(
            &raw,
            "stderr".into(),
            Job::default().stderr_file,
        )?)
    }

    fn _parse_stdout_file(raw: &RawConfig) -> Result<Option<String>> {
        Ok(Self::_parse_one_word_field(
            &raw,
            "stdout".into(),
            Job::default().stderr_file,
        )?)
    }

    fn _parse_stop_wait_seconds(raw: &RawConfig) -> Result<u32> {
        Self::_parse_raw_config_field::<u32>(
            raw,
            String::from("stopwaitsecs"),
            Job::default().stop_wait_secs,
        )
    }

    fn _parse_stop_signal(raw: &RawConfig) -> Result<StopSignals> {
        let field_name: String = String::from("stopsignal");
        match raw.get(&field_name) {
            Some(Some(s)) if *s.to_lowercase() == String::from("hup") => Ok(StopSignals::HUP),
            Some(Some(s)) if *s.to_lowercase() == String::from("int") => Ok(StopSignals::INT),
            Some(Some(s)) if *s.to_lowercase() == String::from("quit") => Ok(StopSignals::QUIT),
            Some(Some(s)) if *s.to_lowercase() == String::from("kill") => Ok(StopSignals::KILL),
            Some(Some(s)) if *s.to_lowercase() == String::from("usr1") => Ok(StopSignals::USR1),
            Some(Some(s)) if *s.to_lowercase() == String::from("usr2") => Ok(StopSignals::USR2),
            Some(Some(s)) if *s.to_lowercase() == String::from("term") => Ok(StopSignals::TERM),
            Some(Some(s)) => Err(Error::FieldBadFormat {
                field_name,
                msg: s.into(),
            }),
            _ => Ok(Job::default().stop_signal),
        }
    }

    fn _parse_start_retries(raw: &RawConfig) -> Result<u32> {
        Self::_parse_raw_config_field::<u32>(
            raw,
            String::from("startretries"),
            Job::default().start_retries,
        )
    }

    fn _parse_start_secs(raw: &RawConfig) -> Result<u32> {
        Self::_parse_raw_config_field::<u32>(
            raw,
            String::from("startsecs"),
            Job::default().start_secs,
        )
    }

    fn _parse_exitcodes(raw: &RawConfig) -> Result<Vec<u8>> {
        let field_name: String = String::from("exitcodes");
        match raw.get(&field_name) {
            Some(Some(str)) => str
                .split(",")
                .map(str::trim)
                .map(|s| {
                    s.parse::<u8>().map_err(|_| Error::CantParseField {
                        field_name: field_name.clone(),
                        value: str.to_string(),
                        type_name: std::any::type_name::<u8>().into(),
                    })
                })
                .collect(),
            _ => Ok(Job::default().exit_codes),
        }
    }

    fn _parse_autorestart(raw: &RawConfig) -> Result<AutorestartOptions> {
        let field_name: String = String::from("autorestart");
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

    fn _parse_num_procs(raw: &RawConfig) -> Result<u32> {
        Self::_parse_raw_config_field::<u32>(
            raw,
            String::from("numprocs"),
            Job::default().num_procs,
        )
    }

    fn _parse_job(raw: &RawConfig) -> Result<Job> {
        Ok(Job {
            command: Self::_parse_command(&raw)?,
            arguments: Self::_parse_arguments(&raw)?,
            num_procs: Self::_parse_num_procs(&raw)?,
            auto_start: Self::_parse_autostart(&raw)?,
            auto_restart: Self::_parse_autorestart(&raw)?,
            exit_codes: Self::_parse_exitcodes(&raw)?,
            start_secs: Self::_parse_start_secs(&raw)?,
            start_retries: Self::_parse_start_retries(&raw)?,
            stop_signal: Self::_parse_stop_signal(&raw)?,
            stop_wait_secs: Self::_parse_stop_wait_seconds(&raw)?,
            stderr_file: Self::_parse_stderr_file(&raw)?,
            stdout_file: Self::_parse_stdout_file(&raw)?,
            environment: Self::_parse_environment(&raw)?,
            work_dir: Self::_parse_working_directory(&raw)?,
            umask: Self::_parse_umask(&raw)?,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
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
                exit_codes: vec![1],
                start_secs: 1,
                start_retries: 3,
                stop_signal: StopSignals::TERM,
                stop_wait_secs: 10,
                stderr_file: None,
                stdout_file: None,
                environment: None,
                work_dir: None,
                umask: None,
                ..Default::default()
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
             command= {command}",
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
                exit_codes: vec![1],
                start_secs: 1,
                start_retries: 3,
                stop_signal: StopSignals::TERM,
                stop_wait_secs: 10,
                stderr_file: None,
                stdout_file: None,
                environment: None,
                work_dir: None,
                umask: None,
                ..Default::default()
            },
        );
        assert_eq!(config.map.len(), 1);
        Ok(())
    }

    #[test]
    fn multiple_config_ok() -> Result<()> {
        let (config_parser, mut config) = get_config_parser_and_config(String::from(
            "[cat]
        command= /bin/cat

        [netcat]
        command=/bin/nc
        numprocs=2
        autostart=true
        autorestart=always
        exitcodes=0, 2, 4
        startsecs=10
        startretries=5
        stopsignal=INT
        stopwaitsecs=20
        stderr=/path/stderr
        stdout=/path/stdout
        environment=FIRSTNAME=\"John\",LASTNAME=\"Doe\"
        workdir=/tmp
        umask=022
",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let mut job: &Job = config.map.get("netcat".into()).unwrap();
        assert_eq!(
            *job,
            Job {
                command: "/bin/nc".into(),
                num_procs: 2,
                auto_start: true,
                auto_restart: AutorestartOptions::Always,
                exit_codes: vec![0, 2, 4],
                start_secs: 10,
                start_retries: 5,
                stop_signal: StopSignals::INT,
                stop_wait_secs: 20,
                stderr_file: Some("/path/stderr".into()),
                stdout_file: Some("/path/stdout".into()),
                environment: Some(HashMap::from([
                    ("FIRSTNAME".into(), "John".into()),
                    ("LASTNAME".into(), "Doe".into())
                ])),
                work_dir: Some("/tmp".into()),
                umask: Some("022".into()),
                ..Default::default()
            },
        );
        job = config.map.get("cat".into()).unwrap();
        assert_eq!(
            *job,
            Job {
                command: "/bin/cat".into(),
                ..Default::default()
            },
        );
        assert_eq!(config.map.len(), 2);
        Ok(())
    }

    #[test]
    fn empty_command() -> Result<()> {
        let (config_parser, mut config) = get_config_parser_and_config(String::from(
            "[test]
             command=",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
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
    fn command_with_arguments() -> Result<()> {
        let (config_parser, mut config) = get_config_parser_and_config(String::from(
            "[test]
             command = nc -nvlp 5555",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get("test".into()).unwrap();
        assert_eq!(
            *job,
            Job {
                command: "nc".into(),
                arguments: Some(vec!("-nvlp".into(), "5555".into())),
                ..Default::default()
            },
        );
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

    #[test]
    fn exitcodes_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             exitcodes=0, 2, 42",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                exit_codes: vec![0, 2, 42],
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn exitcodes_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             exitcodes=1, 2, 5, asdf, 4",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn exitcodes_overflow() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             exitcodes=1, 2, 5, 256, 4",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn start_secs_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             startsecs=30",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                start_secs: 30,
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn start_secs_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             startsecs=bad",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn start_retries_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             startretries=5",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                start_retries: 5,
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn start_retries_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             startretries=bad",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn stop_signals_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stopsignal=INT",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                stop_signal: StopSignals::INT,
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn stop_signal_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stopsignal=bad",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn stop_wait_secs_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stopwaitsecs=20",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                stop_wait_secs: 20,
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn stop_wait_seconds_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stopwaitsecs=bad",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn stderr_file_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stderr=/dev/null",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                stderr_file: Some("/dev/null".to_string()),
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn stderr_file_empty() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stderr=",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                stderr_file: Job::default().stderr_file,
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn stderr_file_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stderr=bad path",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn stdout_file_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stdout=/dev/null",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                stdout_file: Some("/dev/null".to_string()),
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn stdout_file_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             stdout=bad path",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn working_directory_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             workdir=/tmp",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                work_dir: Some("/tmp".to_string()),
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn working_directory_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             workdir=bad path",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn umask_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             umask=012",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                umask: Some("012".to_string()),
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn umask_too_much_char() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             umask=01234",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn umask_bad_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             umask=abc",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn umask_value_too_big() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             umask=019",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn environment_ok() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             environment=A=\"1\",B=\"2\"",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                environment: Some(HashMap::from([
                    ("A".into(), "1".into()),
                    ("B".into(), "2".into())
                ])),
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn environment_nothing_between_comma() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             environment=A=\"1\",,B=\"2\"",
        ));
        let val: Result<()> = config.parse_content_of_parserconfig(config_parser);
        assert!(matches!(val, Err(Error::CantParseEntry { .. })));
        assert!(config.map.is_empty());
        Ok(())
    }

    #[test]
    fn environment_equal_between_double_quotes() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             environment=A=\"1\",B=\"2=5\"",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                environment: Some(HashMap::from([
                    ("A".into(), "1".into()),
                    ("B".into(), "2=5".into())
                ])),
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn environment_quote_in_value() -> Result<()> {
        let job_name: String = String::from("test");
        let command: String = String::from("/bin/test");
        let (config_parser, mut config) = get_config_parser_and_config(format!(
            "[{job_name}]
             command={command}
             environment=A=\"1\",B=\"\"\"",
        ));
        config.parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                environment: Some(HashMap::from([
                    ("A".into(), "1".into()),
                    ("B".into(), "\"".into())
                ])),
                ..Default::default()
            },
        );
        Ok(())
    }
}
