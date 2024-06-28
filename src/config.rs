use crate::parse::parse_job;
use crate::{log, Error, Job, Result};
use configparser::ini::Ini;
use std::collections::HashMap;

pub type ConfigParserContent = HashMap<String, HashMap<String, Option<String>>>;
pub type RawConfig = HashMap<String, Option<String>>;

#[derive(Debug, Clone)]
pub struct Config {
    map: HashMap<String, Job>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            map: HashMap::new(),
        }
    }

    pub fn jobs_routine(&mut self) {
        for (job_name, job) in self.map.iter_mut() {
            job.processes_routine(job_name);
        }
    }

    pub fn get_mut(&mut self, key: &String) -> Option<&mut Job> {
        self.map.get_mut(key)
    }

    pub fn contains_key(&mut self, key: &String) -> bool {
        self.map.contains_key(key)
    }

    pub fn run_autostart_jobs(&mut self) {
        for entry in self.map.iter_mut() {
            let job_name: &String = entry.0;
            let job: &mut Job = entry.1;
            if job.auto_start {
                let _ = job.start(job_name, None);
            }
        }
    }

    pub fn status(&mut self) -> String {
        let mut ret: String = String::new();
        for entry in self.map.iter_mut() {
            let job_name: &String = entry.0;
            let job: &mut Job = entry.1;
            ret = format!("{ret}{}", job.status(job_name, None).unwrap());
        }
        ret
    }

    pub fn reload_config(&mut self, config_path: &String) -> Result<()> {
        let mut new_config: Config = Config::new();
        new_config.parse_config_file(config_path)?;
        for (job_name, new_job) in new_config.map.iter_mut() {
            match self.map.get_mut(job_name) {
                // job is changed case
                Some(old_job) if old_job != new_job => {
                    self.map.get_mut(job_name).unwrap().stop_job_now();
                    self.map.insert(job_name.clone(), new_job.clone());
                    if new_job.auto_start {
                        let job: &mut Job = self.get_mut(job_name).unwrap();
                        let _ = job.start(job_name, None);
                    }
                }
                // job is the same
                Some(old_job) if old_job == new_job => continue,
                // new job case
                _ => {
                    self.map.insert(job_name.clone(), new_job.clone());
                    if new_job.auto_start {
                        let job: &mut Job = self.get_mut(job_name).unwrap();
                        let _ = job.start(job_name, None);
                    }
                    continue;
                }
            };
        }
        // job is deleted
        for (job_name, _old_job) in self.map.clone().iter() {
            if new_config.contains_key(job_name) == false {
                self.map.get_mut(job_name).unwrap().stop_job_now();
                self.map.remove(&job_name.clone());
            }
        }
        Ok(())
    }

    pub fn parse_config_file(&mut self, config_path: &String) -> Result<()> {
        let mut parser: Ini = Ini::new();
        let cfg: ConfigParserContent = parser
            .load(config_path)
            .map_err(|e| Error::CantLoadFile(e.to_string()))?;
        self._parse_content_of_parserconfig(cfg)?;
        Ok(())
    }

    fn _parse_content_of_parserconfig(&mut self, cfg: ConfigParserContent) -> Result<()> {
        for entry in cfg {
            let entry_name: String = entry.0.clone();
            let job: Job = match parse_job(&entry.1) {
                Err(e) => {
                    log(&format!("ERROR: {e}"));
                    return Err(Error::CantParseEntry {
                        entry_name,
                        e: e.to_string(),
                    });
                }
                Ok(content) => content,
            };
            self.map.insert(entry_name, job);
        }
        self.map.remove("default");
        if self
            .map
            .keys()
            .filter(|k| k.chars().all(char::is_alphanumeric))
            .collect::<Vec<&String>>()
            .len()
            != self.map.len()
        {
            return Err(Error::JobEntryNameWithNonAlphanumChar);
        }
        if self.map.is_empty() {
            return Err(Error::NoJobEntry);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::{AutorestartOptions, StopSignals};

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
            config._parse_content_of_parserconfig(config_parser),
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
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
        config._parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                num_procs: 1,
                auto_start: false,
                auto_restart: AutorestartOptions::UnexpectedExit,
                exit_codes: vec![0],
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
        config._parse_content_of_parserconfig(config_parser)?;
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
                umask: Some(0o22),
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
            config._parse_content_of_parserconfig(config_parser),
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
        let job: &Job = config.map.get(&job_name).unwrap();
        assert_eq!(
            *job,
            Job {
                command,
                umask: Some(0o12),
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        let val: Result<()> = config._parse_content_of_parserconfig(config_parser);
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
        config._parse_content_of_parserconfig(config_parser)?;
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
        config._parse_content_of_parserconfig(config_parser)?;
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
