use std::collections::HashMap;
use std::process::{Child, Command, Stdio};

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

// pub struct Child {
//     pub stdin: Option<ChildStdin>,
//     pub stdout: Option<ChildStdout>,
//     pub stderr: Option<ChildStderr>,
//     // some fields omitted
// }

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
    pub is_running: bool,
    // pub process: vec<Child>, // pour store les process
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
            is_running: false,
            // process: None, ////////////
        }
    }
}

// !!! is_running is not check
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
    pub fn start(self: &mut Self, job_name: &String) {
        println!("log: start {}", job_name);

        // Checker si le process CIBLE is running
        // To do : trouver un moyen de cibler le process. Iterer sur les process ?
        // if !(self.process.is_none()) {
        //     print!("Process is not running.");

        // }

        let mut child = Command::new(self.command)
            .stderr(self.stderr_file) // configure the child process's standard error handle
            .stdout(self.stdout_file)
            .spawn()
            .expect("start failed");
        // configure the child process's standard output handle

        let status = child.wait().unwrap();
        // waiting for process to finish

        println!("status : {}", status);
        // self.process.push(child); // ranger le process dans le vecteur process de job
        // // let output = output.stdout;

        self.is_running = true;
    }

    pub fn restart(self: &mut Self, job_name: &String) {
        println!("log: restart {}", job_name);
        self.stop(job_name);
        self.start(job_name);
    }

    pub fn stop(self: &mut Self, job_name: &String) {
        println!("log: stop {}", job_name);
        // checker si le process run
        // self.process.kill().expect("Failed to kill the process");
        self.is_running = false; // Passer a false ?
    }
}
