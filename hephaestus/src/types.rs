use std::fmt;
use std::process::Command;

/// Structure to store information about step
pub struct Step {
    pub step_name: String,
    pub description: String,
    pub step_type: StepType,
    pub action: Option<Action>,
    pub parent: Option<String>,
    pub status: StepStatus,
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
            action: None,
            parent: None,
            status: StepStatus::NotRun,
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

/// Action alias command:
/// - cmd => Command which must be executed
/// - args => Arguments of program
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

/// Type of step:
/// - Action => Regular step
/// - Recovery => Regular step has failed, it is a recovery step for regular step
/// - None => Step type is not set yet
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

/// Enum for step running status:
/// - Ok => Command has run with 0 code
/// - Nok => Command has run with higher than 0 code
/// - Failed => Some internal issue happened
/// - NotRun => Step is waiting for execution
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