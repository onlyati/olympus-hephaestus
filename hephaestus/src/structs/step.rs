use std::collections::HashMap;
use std::process::Command;
use std::path::Path;
use std::io::{Read, BufReader, BufRead};
use std::process::Stdio;

use super::enums::{StepStatus, StepType, StepOutputType};
use super::action::Action;

use chrono::Datelike;
use chrono::Timelike;

#[derive(Clone)]
pub struct Step {
    pub step_name: String,
    pub description: String,
    pub step_type: StepType,
    pub user: Option<String>,
    pub action: Option<Action>,
    pub parent: Option<String>,
    pub status: StepStatus,
    pub envvars: HashMap<String, String>
}

impl Step {
    /// Create new empty step
    /// 
    /// It does not requires any input, but if data is not filled up, it would fail on validate process
    pub fn new_empty() -> Step {
        return Step {
            step_name: String::new(),
            description: String::new(),
            step_type: StepType::None,
            user: None,
            action: None,
            parent: None,
            status: StepStatus::NotRun,
            envvars: HashMap::new(),
        };
    }

    /// Validate step
    /// 
    /// Be assumed that the specified step is okay and correct to run it, it has every mandatory data
    pub fn validate(&self) -> Result<(), String> {
        let mut err_msg: String = String::new();

        if self.step_name.is_empty() {
            err_msg += "Step name cannot be empty!\n";
        }

        if self.description.is_empty() {
            err_msg += "Description cannot be empty!\n";
        }

        if self.step_type == StepType::None {
            err_msg += "Step type must be specified!\n";
        }

        if let None = self.action {
            err_msg += "Action must be specified!\n";
        }

        if self.step_type == StepType::Recovery {
            if let None = self.parent {
                err_msg += "Recovery step must have parent!\n";
            }
        }
        
        if err_msg.is_empty() {
            return Ok(());
        }
        return Err(err_msg);
    }

    /// Execute the command from the step and change its status accordingly
    pub fn execute(&mut self) -> Vec<String> {
        let mut log: Vec<String> = Vec::new();

        match &self.action {
            Some(act) => {
                if act.cmd.len() == 0 {
                    self.status = StepStatus::Failed;
                    return vec!(String::from("Command is not specified"));
                }

                let mut cmd: Command = match &self.user {
                    Some(u) => {
                        let mut b_cmd = Command::new("/usr/bin/sudo");
                        b_cmd.arg("-u");
                        b_cmd.arg(u);
                        b_cmd.arg("bash");
                        b_cmd.arg("-c");
                        b_cmd.arg(act.cmd.join(" "));
                        b_cmd
                    },
                    None => {
                        let mut b_cmd = Command::new("bash");
                        b_cmd.arg("-c");
                        b_cmd.arg(act.cmd.join(" "));
                        b_cmd
                    },
                };

                if self.envvars.len() > 0 {
                    for (key, value) in &self.envvars {
                        cmd.env(key, value);
                    }
                }

                if let Some(cwd) = &act.cwd {
                    let path = Path::new(cwd);
                    if !path.exists() {
                        self.status = StepStatus::Failed;
                        return vec!(format!("Work directory does not exist: {}\n", path.display()));
                    }
                    cmd.current_dir(path);
                }

                let mut child = cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();

                let mut stdout: Vec<StepOutput> = Vec::new();
                let mut stderr: Vec<StepOutput> = Vec::new();

                std::thread::scope(|spawner| {
                    spawner.spawn(|| {
                        let pipe = child.stdout.as_mut().unwrap();
                        stdout = read_buffer(&mut BufReader::new(pipe), StepOutputType::Info);
                    });
                    spawner.spawn(|| {
                        let pipe = child.stderr.as_mut().unwrap();
                        stderr = read_buffer(&mut BufReader::new(pipe), StepOutputType::Error);
                    });
                });

                stdout.append(&mut stderr);
                stdout.sort_by(|a, b| a.time.cmp(&b.time));

                for msg in stdout {
                    log.push(format!("{} {} {}", msg.time, msg.out_type, msg.text));
                }
            }
            None => {
                self.status = StepStatus::Nok;
                return Vec::new();
            }
        };

        return log;
    }
}

struct StepOutput {
    time: String,
    text: String,
    out_type: StepOutputType,
}

// Internal function, it is used to read the stdout and stderr of agent
fn read_buffer<T: Read>(reader: &mut BufReader<T>, out_type: StepOutputType) -> Vec<StepOutput> {
    let mut line = String::new();
    let mut messages: Vec<StepOutput> = Vec::new();

    while let Ok(size) = reader.read_line(&mut line) {
        if size == 0 {
            break;
        }

        let now = chrono::Local::now();
        let now = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
        messages.push(StepOutput { 
            time: now, 
            text: line, 
            out_type: out_type 
        });

        line = String::new();
    }

    return messages;
}