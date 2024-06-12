use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
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

#[derive(Debug)]
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
    pub processes: HashMap<u32, Child>,
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
            processes: HashMap::new(),
        }
    }
}

// clone() seulement sur les types non primitifs donc deep copy
impl Clone for Job {
    fn clone(&self) -> Job {
        Job {
            command: self.command.clone(),
            num_procs: self.num_procs,
            auto_start: self.auto_start,
            auto_restart: self.auto_restart.clone(),
            exit_codes: self.exit_codes.clone(),
            start_secs: self.start_secs,
            start_retries: self.start_retries,
            stop_signal: self.stop_signal.clone(),
            stop_wait_secs: self.stop_wait_secs,
            stderr_file: self.stderr_file.clone(),
            stdout_file: self.stdout_file.clone(),
            environment: self.environment.clone(),
            work_dir: self.work_dir.clone(),
            umask: self.umask.clone(),
            processes: HashMap::new(),
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
    pub fn start(self: &mut Self, job_name: &String) {
        println!("log: start {}", job_name);

        for i in 0..self.num_procs {
            let mut command = Command::new(&self.command);

            if let Some(child) = self.processes.get_mut(&i) {
                match child.try_wait() {
                    Ok(None) => {
                        println!("Process is already running."); // eprint ?
                        continue; // checker le comportement si il est deja run dans la doc
                    }
                    Ok(Some(_)) => {}
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        return;
                    }
                }
            }

            // if let Some(ref config_umask) = self.umask {
            //     match command.pre_exec( || {
            //         unsafe { libc::umask(config_umask) }; // checker si c'est le bon input
            //     }) {
            //         Ok(_) => {}
            //         Err(e) => {
            //             eprintln!("Error: {:?}", e);
            //             return;
            //         }
            //     }
            // }

            if let Some(ref work_dir) = self.work_dir {
                if fs::metadata(work_dir).is_err() {
                    eprintln!("Error: {:?}", work_dir);
                    return;
                }
                command.current_dir(work_dir);
            }

            // checker si ca vaut none aussi dans ce cas on change rien
            if let Some(ref stderr_file) = self.stderr_file {
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(stderr_file)
                {
                    Ok(file) => {
                        command.stderr(Stdio::from(file));
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        // = PermissionDenied
                        return;
                    }
                }
            }

            if let Some(ref stdout_file) = self.stdout_file {
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(stdout_file)
                {
                    Ok(file) => {
                        command.stdout(Stdio::from(file));
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        // = PermissionDenied
                        return;
                    }
                }
            }

            // if let Some(ref arguments) = self.arguments {
            //     match OpenOptions::new().write(true).create(true).open(arguments) {
            //         Ok(file) => {
            // 			command.args(arguments);
            //         }
            //         Err(e) => {
            //             eprintln!("Error: {:?}", e);
            //             return;
            //         }
            //     }
            // }

            // environment

            match command.spawn() {
                Ok(child_process) => {
                    self.processes.insert(i, child_process);
                }
                Err(e) => {
                    eprintln!("Failed to start process: {:?}", e);
                }
            }
        }
    }

    pub fn restart(self: &mut Self, job_name: &String) {
        println!("log: restart {}", job_name);
        self.stop(job_name);
        self.start(job_name);
    }

    pub fn stop(self: &mut Self, job_name: &String) {
        println!("log: stop {}", job_name);
        // if self.is_running == true {
        //     match self.processes.kill() { // preciser quel child je kill
        //         Err(e) => println!("Error: {:?}", e)
        //     }
        // } else {
        //     println!("Process is not running");
        // }
    } // stop signal
}
