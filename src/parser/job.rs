use crate::logger::log;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

use crate::error::{Error, Result};

extern "C" {
    fn kill(pid: u32, signal: i32);
}

extern "C" {
    pub fn umask(mask: u32) -> u32;
}

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
    Stopped,
    Stopping,
    Starting,
    Running,
    Exited,
    Fatal,
    Backoff,
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
            state: ProcessStates::Stopped,
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

    pub fn can_start(self: &Self) -> bool {
        match self.state {
            ProcessStates::Stopped => true,
            ProcessStates::Fatal => true,
            ProcessStates::Exited => true,
            ProcessStates::Backoff => true,
            _ => false,
        }
    }

    pub fn can_stop(self: &Self) -> bool {
        match self.state {
            ProcessStates::Running => true,
            ProcessStates::Starting => true,
            _ => false,
        }
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
    pub umask: Option<u32>,
    pub processes: Vec<ProcessInfo>,
}

impl Default for Job {
    fn default() -> Self {
        Job {
            command: String::new(),
            arguments: None,
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
            processes: vec![ProcessInfo::default()],
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
            processes: vec![ProcessInfo::default(); self.num_procs as usize],
        }
    }
}

impl PartialEq for Job {
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
    pub fn start(
        self: &mut Self,
        job_name: &String,
        target_process: Option<usize>,
    ) -> Result<String> {
        let mut start_index: usize = 0;
        let mut end_index: usize = self.num_procs as usize;
        if let Some(nb) = target_process {
            if nb < self.num_procs as usize {
                start_index = nb;
                end_index = nb + 1;
            } else {
                return Err(Error::StartJobFail(format!(
                    "Target index must be inferior to {:?}",
                    self.num_procs
                )));
            }
        }

        for i in start_index..end_index {
            if self.processes[i].can_start() == false {
                log(&format!(
                    "{job_name}:{i} is in a state where it can't start"
                ));
                eprintln!("{job_name}:{i} is in a state where it can't start");
                continue;
            }
            let mut command = Command::new(&self.command);
            if let Some(args) = &self.arguments {
                command.args(args);
            }

            if let Some(environment) = &self.environment {
                command.envs(environment);
            }

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
                    return Err(Error::StartJobFail(format!(
                        "{work_dir} is not a directory!"
                    )));
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
                    Err(e) => return Err(Error::StartJobFail(e.to_string())),
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
                    Err(e) => return Err(Error::StartJobFail(e.to_string())),
                }
            }

            match command.spawn() {
                Ok(child_process) => {
                    self.processes[i as usize].nb_retries += 1;
                    self.processes[i as usize].child = Some(child_process);
                    self.processes[i as usize].set_state(ProcessStates::Starting);
                    log(&format!("{job_name}:{i} is now in STARTING state"));
                    println!("LOG: {job_name}:{i} is now in STARTING state");
                }
                Err(e) => return Err(Error::StartJobFail(e.to_string())),
            }
        }
        Ok(format!("{job_name} is started successfully!"))
    }

    pub fn restart(
        self: &mut Self,
        job_name: &String,
        target_process: Option<usize>,
    ) -> Result<String> {
        println!("LOG: restart {}", job_name);
        self.stop(job_name, target_process)?;
        self.start(job_name, target_process)?;
        Ok(format!("{job_name} is restarted successfully!"))
    }

    pub fn stop(
        self: &mut Self,
        job_name: &String,
        target_process: Option<usize>,
    ) -> Result<String> {
        let mut start_index: usize = 0;
        let mut end_index: usize = self.num_procs as usize;
        if let Some(nb) = target_process {
            if nb < self.num_procs as usize {
                start_index = nb;
                end_index = nb + 1;
            } else {
                return Err(Error::StopJobFail(format!(
                    "Target index must be inferior to {}",
                    self.num_procs
                )));
            }
        }
        for i in start_index..end_index {
            let process: &mut ProcessInfo = &mut self.processes[i as usize];
            if process.can_stop() == false {
                log(&format!("{job_name}:{i} is in a state where it can't stop"));
                eprintln!("{job_name}:{i} is in a state where it can't stop");
                continue;
            }
            let child_id: u32 = process.child.as_ref().unwrap().id();
            unsafe {
                kill(child_id, self.stop_signal.to_owned() as i32);
            }
            self.processes[i].set_state(ProcessStates::Stopping);
            log(&format!("{job_name}:{i} is now in STOPPING state"));
            println!("LOG: {job_name}:{i} is now in STOPPING state");
        }
        Ok(format!("{job_name} is stopped successfully!"))
    }

    pub fn stop_job_now(self: &mut Self) {
        for p in self.processes.iter_mut() {
            if let Some(c) = &mut p.child {
                let _ = c.kill();
                let _ = c.wait();
            }
        }
    }

    // from http://supervisord.org/subprocess.html#process-states
    pub fn processes_routine(self: &mut Self, job_name: &String) {
        let nb_processes: usize = self.num_procs as usize;
        for process_index in 0..nb_processes {
            match self.processes[process_index].state {
                ProcessStates::Starting => self._handle_starting(process_index, job_name),
                ProcessStates::Backoff => self._handle_backoff(process_index, job_name),
                ProcessStates::Stopping => self._handle_stopping(process_index, job_name),
                ProcessStates::Running => self._handle_running(process_index, job_name),
                ProcessStates::Exited => self._handle_exited(process_index, job_name),
                // fatal and stopped need user interaction to change
                _ => continue,
            };
        }
    }

    fn _handle_starting(self: &mut Self, process_index: usize, job_name: &String) {
        let process: &mut ProcessInfo = &mut self.processes[process_index];
        let child: &mut Child = if let Some(c) = &mut process.child {
            c
        } else {
            log(&format!(
                "{job_name}:{process_index} Unexpected error while starting"
            ));
            panic!("Why process state is STARTING but child is NONE ????");
        };
        match child.try_wait() {
            Ok(Some(_)) => {
                process.set_state(ProcessStates::Backoff);
                process.child = None;
                println!("LOG: {job_name}:{process_index} is now in BACKOFF state");
            }
            Ok(None) => {
                if process.state_changed_at.elapsed().as_secs() >= self.start_secs as u64 {
                    process.nb_retries = 0;
                    process.set_state(ProcessStates::Running);
                    log(&format!(
                        "{job_name}:{process_index} is now in RUNNING state"
                    ));
                    println!("LOG: {job_name}:{process_index} is now in RUNNING state");
                }
            }
            Err(e) => {
                log(&format!(
                    "{job_name}:{process_index} Error attempting to wait: {e}"
                ));
            }
        }
    }

    fn _handle_backoff(&mut self, process_index: usize, job_name: &String) {
        let process: &mut ProcessInfo = &mut self.processes[process_index];
        if process.nb_retries < self.start_retries {
            if (process.state_changed_at.elapsed().as_secs() as u32) < process.nb_retries {
                return;
            }
            let _ = self.start(job_name, Some(process_index));
            return;
        }
        process.nb_retries = 0;
        process.child = None;
        process.set_state(ProcessStates::Fatal);
        log(&format!("{job_name}:{process_index} is now in FATAL state"));
        println!("LOG: {job_name}:{process_index} is now in FATAL state");
    }

    fn _handle_stopping(&mut self, process_index: usize, job_name: &String) {
        let process: &mut ProcessInfo = &mut self.processes[process_index];
        let child: &mut Child = if let Some(c) = &mut process.child {
            c
        } else {
            log(&format!(
                "{job_name}:{process_index} Unexpected error while stopping"
            ));
            panic!("Why process state is STOPPING but child is NONE ????");
        };
        match child.try_wait() {
            Ok(Some(_)) => {
                process.set_state(ProcessStates::Stopped);
                process.child = None;
                log(&format!(
                    "{job_name}:{process_index} is now in STOPPED state"
                ));
                println!("LOG: {job_name}:{process_index} is now in STOPPED state");
            }
            Ok(None) => {
                if process.state_changed_at.elapsed().as_secs() >= self.stop_wait_secs as u64 {
                    let _ = child.kill();
                }
            }
            Err(e) => {
                log(&format!(
                    "{job_name}:{process_index} Error attempting to wait: {e}"
                ));
            }
        }
    }

    fn _handle_running(&mut self, process_index: usize, job_name: &String) {
        let process: &mut ProcessInfo = &mut self.processes[process_index];
        let child: &mut Child = if let Some(c) = &mut process.child {
            c
        } else {
            log(&format!(
                "{job_name}:{process_index} Unexpected error while running"
            ));
            panic!("Why process state is RUNNING but child is NONE ????");
        };
        match child.try_wait() {
            Ok(Some(status)) if status.code().is_none() => {
                // terminated by signal
                process.set_state(ProcessStates::Stopped);
                log(&format!(
                    "{job_name}:{process_index} is now in STOPPED state"
                ));
                println!("LOG: {job_name}:{process_index} is now in STOPPED state");
            }
            Ok(Some(_)) => {
                process.set_state(ProcessStates::Exited);
                log(&format!(
                    "{job_name}:{process_index} is now in EXITED state"
                ));
                println!("LOG: {job_name}:{process_index} is now in EXITED state");
            }
            Err(e) => {
                log(&format!(
                    "{job_name}:{process_index} Error attempting to wait: {e}"
                ));
            }
            _ => return,
        }
    }

    fn _handle_exited(&mut self, process_index: usize, job_name: &String) {
        let process: &mut ProcessInfo = &mut self.processes[process_index];
        let child: &mut Child = if let Some(c) = &mut process.child {
            c
        } else {
            return;
        };
        match child.try_wait() {
            Ok(Some(status)) if status.code().is_none() => {
                log(&format!(
                    "{job_name}:{process_index} Unexpected error while exiting"
                ));
                panic!("Why process state is EXITED but it's terminated by SIGNAL ????");
            }
            Ok(Some(status)) => {
                // is safe: process can't be in exited state and terminated by signal
                let code: i32 = status.code().unwrap();
                if self.auto_restart == AutorestartOptions::Always
                    || (self.auto_restart == AutorestartOptions::UnexpectedExit
                        && self.exit_codes.contains(&code) == false)
                {
                    let _ = self.start(job_name, Some(process_index));
                }
            }
            Err(e) => eprintln!("Error attempting to wait: {e}"),
            Ok(None) => {
                log(&format!(
                    "{job_name}:{process_index} Unexpected error while exiting"
                ));
                panic!("Why process state is EXITED but process is not TERMINATED ????")
            }
        }
    }
}
