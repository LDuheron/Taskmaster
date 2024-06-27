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
            Ok(split_cmd.to_string())
        } else {
            Ok(cmd.to_string())
        }
    } else {
        Err(Error::WrongClientInputFormat)
    }
}

fn parse_target_process_number_from_client_input(raw: &String) -> Result<Option<usize>> {
    if let Some(cmd) = raw.split_whitespace().skip(1).next() {
        if let Some(index) = cmd.rfind(":") {
            let split_cmd = &cmd[index + 1..];
            if let Ok(number) = split_cmd.parse::<usize>() {
                return Ok(Some(number));
            } else {
                return Err(Error::WrongClientInputFormat);
            }
        }
    }
    Ok(None)
}

    fn parse_content_of_parserconfig(&mut self, cfg: ConfigParserContent) -> Result<()> {
        for entry in cfg {
            let entry_name: String = entry.0.clone();
            let job: Job = match Self::_parse_job(&entry.1) {
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

    pub fn parse_config_file(&mut self, config_path: &String) -> Result<()> {
        let mut parser: Ini = Ini::new();
        let cfg: ConfigParserContent = parser
            .load(config_path)
            .map_err(|e| Error::CantLoadFile(e.to_string()))?;
        self.parse_content_of_parserconfig(cfg)?;
        Ok(())
    }

    //
    // -- PRIVATE
    //

    fn _parse_raw_config_field<T: FromStr>(
        raw: &RawConfig,
        field_name: String,
        default: T,
    ) -> Result<T> {
        match raw.get(&field_name) {
            Some(Some(value)) => Ok(value.parse::<T>().map_err(|_| Error::CantParseField {
                field_name,
                value: value.to_string(),
                type_name: type_name::<T>().into(),
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
        if is_valid_umask == false {
            return Err(Error::FieldBadFormat {
                field_name,
                msg: "Field contain too much characters".into(),
            });
        }
        let umask = u32::from_str_radix(&umask_str, 8);
        match umask {
            Ok(umask) => Ok(Some(umask)),
            Err(_) => Err(Error::CantParseField {
                field_name,
                value: umask_str,
                type_name: type_name::<u32>().into(),
            }),
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

    fn _parse_exitcodes(raw: &RawConfig) -> Result<Vec<i32>> {
        let field_name: String = String::from("exitcodes");
        match raw.get(&field_name) {
            Some(Some(str)) => str
                .split(",")
                .map(str::trim)
                .map(|s| {
                    s.parse::<i32>().map_err(|_| Error::CantParseField {
                        field_name: field_name.clone(),
                        value: str.to_string(),
                        type_name: type_name::<i32>().into(),
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
        let num_procs: u32 = Self::_parse_num_procs(&raw)?;
        Ok(Job {
            command: Self::_parse_command(&raw)?,
            arguments: Self::_parse_arguments(&raw)?,
            num_procs,
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
            processes: vec![ProcessInfo::default(); num_procs as usize],
        })
    }
}

