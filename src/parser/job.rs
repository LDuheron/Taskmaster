use std::collections::HashMap;
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};

extern "C" {
    pub fn umask(mask: u32) -> u32;
}

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
    pub arguments: Option<Vec<String>>,
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
    pub umask: Option<u32>,
    // TODO
    pub processes: Option<HashMap<u32, Child>>,
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
            // TODO
            processes: None,
        }
    }
}

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
            umask: self.umask,
            // TODO
            processes: None,
        }
    }
}

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
        // TODO
        // && self.processes == other.processes
    }
}

impl Job {
    pub fn start(self: &mut Self, job_name: &String, target_process: Option<u32>) {
        println!("log: start {}", job_name);

        let mut start_index = 0;
        let mut end_index = self.num_procs;
        if let Some(nb) = target_process {
            if nb < self.num_procs {
                start_index = nb;
                end_index = nb + 1;
            } else {
                eprintln!(
                    "Target index must be inferior or equal to {:?}",
                    self.num_procs
                );
                return;
            }
        }

        for i in start_index..end_index {
            let mut command = Command::new(&self.command);

            // TODO : modifier le if pour cibler le process[i]
            if let Some(args) = &self.arguments {
                command.args(args);
            }

            if let Some(environment) = &self.environment {
                command.envs(environment);
            }

            // Move -> prend l'ownership de config_umask
            if let Some(config_umask) = self.umask {
                unsafe {
                    command.pre_exec(move || {
                        umask(config_umask);
                        Ok(())
                    });
                }
            }

            if let Some(ref work_dir) = self.work_dir {
                let path = Path::new(work_dir);
                if path.is_dir() == true {
                    command.current_dir(work_dir);
                } else {
                    eprintln!("Error: {:?}", work_dir);
                    return;
                }
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
                    if let Some(ref mut map) = self.processes {
                        map.insert(i, child_process);
                        println!();
                    } else {
                        // TODO : modify to the new vector
                        let mut map: HashMap<u32, Child> = HashMap::new();
                        map.insert(i, child_process);
                        self.processes = Some(map);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to start process: {:?}", e);
                }
            }
        }
    }

    pub fn restart(self: &mut Self, job_name: &String, target_process: Option<u32>) {
        println!("log: restart {}", job_name);
        self.stop(job_name, target_process);
        self.start(job_name, target_process);
    }

    pub fn stop(self: &mut Self, job_name: &String, target_process: Option<u32>) {
        println!("log: stop {}", job_name);

        let mut start_index = 0;
        let mut end_index = self.num_procs;
        if let Some(nb) = target_process {
            if nb < self.num_procs {
                start_index = nb;
                end_index = nb + 1;
            } else {
                eprintln!(
                    "Target index must be inferior or equal to {:?}",
                    self.num_procs
                );
                return;
            }
        }

        if let Some(ref mut map) = self.processes {
            for i in start_index..end_index {
                if let Some(mut child) = map.remove(&i) {
                    println!("Process is running.");
                    // Functional version
                    match child.kill() {
                        Ok(_) => {
                            println!("Process is dead.");
                        }
                        Err(e) => {
                            eprint!("{:?}", e)
                        }
                    }

                    // let mut child_id: u32 = child.id();

                    // if let Some(mut signal) = self.stop_signal {
                    //     unsafe {
                    //         kill(child_id, signal);
                    //     }
                    // }
                    // else {
                    // 	unsafe {
                    //         kill(child_id, SIGTERM);
                    //     }
                    // }
                    // map.remove(&0);
                }
            }
        }
    }
}
