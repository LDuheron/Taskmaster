use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum AutorestartOptions {
    Always,
    Never,
    UnexpectedExit,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum StopSignals {
    HUP = 1,
    INT = 2,
    QUIT = 3,
    KILL = 9,
    USR1 = 10,
    USR2 = 12,
    TERM = 15,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub command: String,
    pub num_procs: u32,
    pub auto_start: bool,
    pub auto_restart: AutorestartOptions,
    pub exit_codes: Vec<u8>,
    pub start_secs: u32,
    pub start_retries: u32,
    pub stop_signal: StopSignals,
    pub stop_wait_secs: u32,
    pub stderr_file: Option<String>,
    pub stdout_file: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub work_dir: Option<String>,
    pub umask: Option<String>,
}

impl Default for Job {
    fn default() -> Self {
        Job {
            command: String::new(),
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
        }
    }
}

impl std::cmp::PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
            && self.num_procs == other.num_procs
            && self.auto_start == other.auto_start
            && self.auto_restart == other.auto_restart
            && self.exit_codes == other.exit_codes
            && self.start_secs == other.start_secs
            && self.start_retries == other.start_retries
            && self.stop_signal == other.stop_signal
            && self.stop_wait_secs == other.stop_wait_secs
            && self.stderr_file == other.stderr_file
            && self.stdout_file == other.stdout_file
            && self.environment == other.environment
            && self.work_dir == other.work_dir
            && self.umask == other.umask
    }
}

impl Job {
    pub fn start(self: &Self, job_name: &String) {
        println!("start {}", job_name);
    }

    pub fn restart(self: &Self, job_name: &String) {
        println!("restart {}", job_name);
    }

    pub fn stop(self: &Self, job_name: &String) {
        println!("stop {}", job_name);
    }
}
