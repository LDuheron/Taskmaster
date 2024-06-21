use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::Path;
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
    // pub name: String,
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
    pub umask: Option<String>,
    // TODO
    pub processes: Option<HashMap<u32, Child>>,
}

impl Default for Job {
    fn default() -> Self {
        Job {
            // name: String::new(),
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
            // name: self.name.clone(),
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
            processes: None, // TODO
        }
    }
}

impl std::cmp::PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        // self.name == other.name
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
    pub fn start(self: &mut Self, job_name: &String) {
        println!("log: start {}", job_name);

        for i in 0..self.num_procs {
            let mut command = Command::new(&self.command);

            // TODO : modifier le if pour cibler le process[i]
            if let Some(args) = &self.arguments {
                command.args(args);
            }

            if let Some(environment) = &self.environment {
                command.envs(environment);
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

    pub fn restart(self: &mut Self, job_name: &String) {
        println!("log: restart {}", job_name);
        self.stop(job_name);
        self.start(job_name);
    }

    pub fn stop(self: &mut Self, job_name: &String) {
        println!("log: stop {}", job_name);

        if let Some(ref mut map) = self.processes {
            if let Some(child) = map.get_mut(&0) {
                println!("Process is running.");
                // Functional version version
                let _ = child.kill();
                map.remove(&0);

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

// TODO
// tester si le job appartient ou non a la hashmap<>
//
