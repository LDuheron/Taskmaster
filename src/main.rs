use configparser::ini::Ini;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
enum AutorestartOptions {
    Always,
    Never,
    UnexpectedExit,
}

#[derive(Debug)]
enum Signals {
    Undefined,
    Hup,
    Int,
    Quit,
    Ill,
    Trap,
    Iot,
    Bus,
    Fpe,
    Kill,
    Usr1,
    Segv,
    Usr2,
    Pipe,
    Alrm,
    Term,
    Stkflt,
    Chld,
    Cont,
    Stop,
    Tstp,
    Ttin,
    Ttou,
    Urg,
    Xcpu,
    Xfsz,
    Vtalrm,
    Prof,
    Winch,
    Poll,
    Pwr,
    Sys,
}

struct Config {
    map: HashMap<String, ConfigEntry>,
}

#[derive(Debug)]
struct ConfigEntry {
    command: String,
    num_procs: u32,
    auto_start: bool,
    auto_restart: AutorestartOptions,
    expected_return_codes: Vec<u32>,
    start_secs: u32,
    start_retries: u32,
    stop_signal: Vec<Signals>,
    stop_wait_secs: u32,
    stdin_file: String,
    stdout_file: String,
    environment: HashMap<String, String>,
    work_dir: String,
    umask: String,
}

#[test]
fn test_func() -> Result<(), String> {
    Err("This is an error".to_string())
}

impl Config {
    fn new() -> Self {
        Config {
            map: HashMap::new(),
        }
    }

    fn parse_config_file(&mut self, config_path: String) -> Result<(), String> {
        let mut parser = Ini::new();
        let cfg = parser.load(config_path)?;
        for entry in &cfg {
            println!("{:?}", entry);
        }
        self.map.insert(
            String::from("cat"),
            ConfigEntry {
                command: String::from("/bin/cat"),
                num_procs: 1,
                auto_start: true,
                auto_restart: AutorestartOptions::Always,
                expected_return_codes: vec![1],
                start_secs: 10,
                start_retries: 3,
                stop_signal: vec![Signals::Term],
                stop_wait_secs: 20,
                stdin_file: String::new(),
                stdout_file: String::new(),
                environment: HashMap::new(),
                work_dir: String::from("/tmp"),
                umask: String::from("032"),
            },
        );
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut config: Config = Config::new();
    config.parse_config_file(String::from("config.ini"))?;
    println!("Config: {:?}", config.map);
    Ok(())
}
