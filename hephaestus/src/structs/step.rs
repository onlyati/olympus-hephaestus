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
    pub fn execute(&mut self) -> Vec<StepOutput> {
        let mut log: Vec<StepOutput>;

        match &self.action {
            Some(act) => {
                if act.cmd.len() == 0 {
                    self.status = StepStatus::Failed;
                    return vec!(StepOutput {
                        time: time_is_now(),
                        text: "Command is not specified".to_string(),
                        out_type: StepOutputType::Error,
                    });
                }

                // Prepare command
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
                        return vec!(StepOutput {
                            time: time_is_now(),
                            text: format!("Work directory does not exist: {}\n", path.display()),
                            out_type: StepOutputType::Error,
                        });
                    }
                    cmd.current_dir(path);
                }

                // Because accurate timestamp is needed, what the program wrote and when, this child has to be spawn
                // Outputs are directed to pipes, which is conitnousily read later
                let mut child = cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();

                let mut stdout: Vec<StepOutput> = Vec::new();
                let mut stderr: Vec<StepOutput> = Vec::new();

                // Connect to child outputs and read them continousily
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

                // Merge stdout and stderr, get the exit code, then return
                stdout.append(&mut stderr);
                stdout.sort_by(|a, b| a.time.cmp(&b.time));
                for line in &mut stdout {
                    line.text = line.text.replace("\n", "");
                }
                log = stdout;


                let status = child.wait();
                match status {
                    Ok(code) => {
                        if code.success() {
                            self.status = StepStatus::Ok;
                            log.push(StepOutput {
                                time: time_is_now(),
                                text: String::from("----> Step is ended with exit code 0"),
                                out_type: StepOutputType::Info,
                            });
                        }
                        else {
                            self.status = StepStatus::Nok;
                            log.push(StepOutput {
                                time: time_is_now(),
                                text: format!("----> Step is ended with exit code {:?}", code.code()),
                                out_type: StepOutputType::Error,
                            });
                        }
                        
                    },
                    Err(e) => { 
                        self.status = StepStatus::Failed;
                        log.push(StepOutput {
                            time: time_is_now(),
                            text: format!("----> Step is failed: {:?}", e),
                            out_type: StepOutputType::Error,
                        });
                    },
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

pub struct StepOutput {
    pub time: String,
    pub text: String,
    pub out_type: StepOutputType,
}

// Internal function, it is used to read the stdout and stderr of agent
fn read_buffer<T: Read>(reader: &mut BufReader<T>, out_type: StepOutputType) -> Vec<StepOutput> {
    let mut line = String::new();
    let mut messages: Vec<StepOutput> = Vec::new();

    while let Ok(size) = reader.read_line(&mut line) {
        if size == 0 {
            break;
        }

        messages.push(StepOutput { 
            time: time_is_now(), 
            text: line,
            out_type: out_type 
        });

        line = String::new();
    }

    return messages;
}

fn time_is_now() -> String {
    let now = chrono::Local::now();
    return format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
}