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

#[derive(Debug)]
struct ConfigEntry {
    program_name: String,
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

fn parse_config_file(
    config_path: String,
) -> Result<HashMap<String, HashMap<String, Option<String>>>, String> {
    let mut parser = Ini::new();
    let cfg = parser.load(config_path)?;
    for entry in &cfg {
        println!("{:?}", entry);
        // println!("{:?}", entry.1["test"]);
    }
    Ok(cfg)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = parse_config_file(String::from("config.ini"))?;
    Ok(())
}
