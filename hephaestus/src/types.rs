use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::process::Command;

pub struct Step {
    pub step_name: String,
    pub description: String,
    pub step_type: StepType,
    pub action: Option<Action>,
    pub parent: Option<Rc<RefCell<Step>>>,
    pub status: StepStatus,
}

impl Step {
    pub fn new_empty() -> Step {
        return Step {
            step_name: String::new(),
            description: String::new(),
            step_type: StepType::None,
            action: None,
            parent: None,
            status: StepStatus::NotRun,
        };
    }

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
        
        if err_msg.is_empty() {
            return Ok(());
        }
        return Err(err_msg);
    }

    pub fn execute(&mut self) {
        match &self.action {
            Some(act) => {
                let cmd = match &act.cmd {
                    Some(v) => v.clone(),
                    None => {
                        self.status = StepStatus::Nok;
                        return;        
                    } 
                };

                let mut cmd = Command::new(cmd);

                for arg in &act.args {
                    cmd.arg(arg);
                }

                match cmd.output() {
                    Ok(o) => {
                        if o.status.success() {
                            self.status = StepStatus::Ok;
                        }
                        else {
                            self.status = StepStatus::Nok;
                        }
                    },
                    Err(_) => self.status = StepStatus::Failed,
                }
            }
            None => {
                self.status = StepStatus::Nok;
                return;
            }
        };
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

pub struct Action {
    pub cmd: Option<String>,
    pub args: Vec<String>,
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
         .field("cmd", &self.cmd)
         .field("args", &self.args)
         .finish()
    }
}

impl Action {
    pub fn new(cmd: String) -> Action {
        return Action {
            cmd: Some(cmd),
            args: Vec::new(),
        }
    }

    pub fn add_arg(&mut self, arg: String) {
        self.args.push(arg);
    }
}

#[derive(Eq, PartialEq)]
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

#[derive(Eq, PartialEq)]
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