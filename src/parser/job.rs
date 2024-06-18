use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

#[derive(Debug, PartialEq, Clone)]
pub enum AutorestartOptions {
    Always,
    Never,
    UnexpectedExit,
}

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

// http://supervisord.org/subprocess.html#process-states
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProcessStates {
    STOPPED,
    STOPPING,
    STARTING,
    RUNNING,
    EXITED,
    FATAL,
    BACKOFF,
}

#[derive(Debug)]
pub struct ProcessInfo {
    pub child: Option<Child>,
    pub state_changed_at: Instant,
    pub state: ProcessStates,
    pub nb_retries: u32,
}

impl Default for ProcessInfo {
    fn default() -> Self {
        ProcessInfo {
            child: None,
            state_changed_at: Instant::now(),
            state: ProcessStates::STOPPED,
            nb_retries: 0,
        }
    }
}

impl Clone for ProcessInfo {
    fn clone(&self) -> Self {
        ProcessInfo {
            child: None,
            state_changed_at: self.state_changed_at,
            state: self.state,
            nb_retries: 0,
        }
    }
}

impl ProcessInfo {
    pub fn set_state(self: &mut Self, state: ProcessStates) {
        self.state = state;
        self.state_changed_at = Instant::now();
    }
}

#[derive(Debug)]
pub struct Job {
    pub command: String,
    pub arguments: Option<Vec<String>>,
    pub num_procs: u32,
    pub auto_start: bool,
    pub auto_restart: AutorestartOptions,
    pub exit_codes: Vec<i32>,
    pub start_secs: u32,
    pub start_retries: u32,
    pub stop_signal: StopSignals,
    pub stop_wait_secs: u32,
    pub stderr_file: Option<String>,
    pub stdout_file: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub work_dir: Option<String>,
    pub umask: Option<String>,
    pub processes: Vec<ProcessInfo>,
}

impl Default for Job {
    fn default() -> Self {
        Job {
            command: String::new(),
            arguments: None,
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
            processes: vec![ProcessInfo::default()],
        }
    }
}

// clone() seulement sur les types non primitifs donc deep copy
impl Clone for Job {
    fn clone(&self) -> Job {
        Job {
            command: self.command.clone(),
            arguments: self.arguments.clone(),
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
            processes: vec![ProcessInfo::default(); self.num_procs as usize],
        }
    }
}

// this don't check for processes
impl std::cmp::PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
            && self.arguments == other.arguments
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
    // TODO: specify the process number
    pub fn start(self: &mut Self, job_name: &String) {
        println!("log: start {}", job_name);

        for i in 0..self.num_procs {
            // something like that to avoid overriding process
            if self.processes[i as usize].state != ProcessStates::STOPPED {
                return;
            }
            let mut command = Command::new(&self.command);

            if let Some(args) = &self.arguments {
                for element in args.iter() {
                    command.arg(element);
                }
            }

            //             if let Some(child) = self.processes.get_mut(&i) {
            //                 match child.try_wait() {
            //                     Ok(None) => {
            //                         println!("Process is already running.");
            //                         continue;
            //                     }
            //                     Ok(Some(_)) => {}
            //                     Err(e) => {
            //                         eprintln!("Error: {:?}", e);
            //                         return;
            //                     }
            //                 }
            //             }

            if let Some(environment) = &self.environment {
                for (key, value) in environment {
                    command.env(key, value);
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
                        return;
                    }
                }
            }

            match command.spawn() {
                Ok(child_process) => {
                    self.processes[i as usize].child = Some(child_process);
                    self.processes[i as usize].set_state(ProcessStates::STARTING);
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
        // if let Some(child) = self.processes.get_mut(&i) {
        //     match child.try_wait() {
        //         Ok(None) => {
        //             println!("Process is running.");
        //             child.kill(); // preciser la facon de kill avec self.stop_sign
        //         }
        //         Ok(Some(_)) => {}
        //         Err(e) => {
        //             eprintln!("Error: {:?}", e);
        //             return;
        //         }
        //     }
        // }
    }

    pub fn processes_routine(self: &mut Self, job_name: &String) {
        let nb_processes: usize = self.num_procs as usize;
        for i in 0..nb_processes {
            match self.processes[i].state {
                ProcessStates::STARTING => self._handle_starting(i, job_name),
                ProcessStates::STOPPING => self._handle_stopping(i, job_name),
                _ => return,
            };
        }
    }

    fn _handle_starting(self: &mut Self, proc_index: usize, job_name: &String) {
        let process = &mut self.processes[proc_index];
        let child: &mut Child = if let Some(c) = &mut process.child {
            c
        } else {
            panic!("Why process state is STARTING but child is NONE ????");
        };
        match child.try_wait() {
            Ok(Some(status)) => match self.auto_restart {
                AutorestartOptions::Always if process.nb_retries <= self.start_retries => {
                    // process is terminated by signal
                    if status.code().is_none() {
                        self.processes[proc_index] = ProcessInfo::default();
                        return;
                    }
                    process.nb_retries += 1;
                    process.state = ProcessStates::BACKOFF;
                    println!("LOG: {job_name}:{proc_index} is now in BACKOFF state");
                    // TODO: specify the process number
                    self.start(job_name);
                }
                AutorestartOptions::UnexpectedExit if process.nb_retries <= self.start_retries => {
                    if let Some(code) = status.code() {
                        if !self.exit_codes.contains(&code) {
                            process.nb_retries += 1;
                            process.state = ProcessStates::BACKOFF;
                            println!("LOG: {job_name}:{proc_index} is now in BACKOFF state");
                            // TODO: specify the process number
                            self.start(job_name);
                        }
                    } else {
                        // process is terminated by signal
                        self.processes[proc_index] = ProcessInfo::default();
                    }
                }
                // process can't autorestart
                _ => self.processes[proc_index] = ProcessInfo::default(),
            },
            Ok(None) => {
                if process.state_changed_at.elapsed().as_secs() >= self.start_secs as u64 {
                    process.set_state(ProcessStates::RUNNING);
                    println!("LOG: {job_name}:{proc_index} is now in RUNNING state");
                }
            }
            Err(e) => println!("error attempting to wait: {e}"),
        }
    }

    fn _handle_stopping(&mut self, proc_index: usize, job_name: &String) {
        let process = &mut self.processes[proc_index];
        todo!();
    }
}
