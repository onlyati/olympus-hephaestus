use std::fmt;
use std::process::Command;
use std::path::Path;
use std::collections::HashMap;

pub struct Plan {
    pub id: String,
    pub status: StepStatus,
    pub steps: Vec<Step>,
}

impl Plan {
    pub fn new(id: String, steps: Vec<Step>) -> Plan {
        return Plan { 
            id: id, 
            status: StepStatus::NotRun,
            steps: steps 
        }
    }
}

/// Structure to store information about step
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
    pub fn execute(&mut self) -> Option<String> {
        let mut text: Option<String> = None;

        match &self.action {
            Some(act) => {
                if act.cmd.len() == 0 {
                    self.status = StepStatus::Failed;
                    return Some(String::from("Command is not specified"));
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
                        return Some(format!("Work direcotry does not exist: {}\n", path.display()));
                    }
                    cmd.current_dir(path);
                }

                match cmd.output() {
                    Ok(o) => {
                        if o.status.success() {
                            self.status = StepStatus::Ok;
                        }
                        else {
                            self.status = StepStatus::Nok;
                        }

                        text = match String::from_utf8(o.stdout) {
                            Ok(r) => Some(r),
                            Err(_) => None,
                        };

                        match String::from_utf8(o.stderr) {
                            Ok(r) => {
                                match text {
                                    Some(ref mut v) => *v += &r[..],
                                    None => text = Some(r),
                                }
                            },
                            Err(_) => (),
                        }
                    },
                    Err(_) => self.status = StepStatus::Failed,
                }
            }
            None => {
                self.status = StepStatus::Nok;
                return None;
            }
        };

        return text;
    }
}

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Step")
         .field("step_name", &self.step_name)
         .field("description", &self.description)
         .field("step_type", &self.step_type)
         .field("action", &self.action)
         .field("parent", &self.parent)
         .finish()
    }
}

/// Action alias command:
/// - cmd => Command which must be executed
/// - args => Arguments of program
#[derive(Clone)]
pub struct Action {
    pub cmd: Vec<String>,
    pub cwd: Option<String>,
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
         .field("cmd", &self.cmd)
         .field("cwd", &self.cwd)
         .finish()
    }
}

impl Action {
    pub fn new(cmd: String, cwd: Option<String>) -> Action {
        let base = vec![String::from(cmd)];
        return Action {
            cmd: base,
            cwd: cwd,
        }
    }
}

/// Type of step:
/// - Action => Regular step
/// - Recovery => Regular step has failed, it is a recovery step for regular step
/// - None => Step type is not set yet
#[derive(Eq, PartialEq, Clone)]
pub enum StepType {
    Action,
    Recovery,
    None,
}

impl fmt::Debug for StepType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printable = match *self {
            StepType::Action => "step",
            StepType::Recovery => "recovery",
            StepType::None => "not specified",
        };
        write!(f, "{}", printable)
    }
}

/// Enum for step running status:
/// - Ok => Command has run with 0 code
/// - Nok => Command has run with higher than 0 code
/// - Failed => Some internal issue happened
/// - NotRun => Step is waiting for execution
#[derive(Eq, PartialEq, Clone)]
pub enum StepStatus {
    Ok,
    Nok,
    NotRun,
    Failed,
}

impl fmt::Debug for StepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printable = match *self {
            StepStatus::Ok => "OK",
            StepStatus::Nok => "NOK",
            StepStatus::NotRun => "Did not run",
            StepStatus::Failed => "Failed",
        };
        write!(f, "{}", printable)
    }
}