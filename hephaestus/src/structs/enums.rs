use std::fmt;

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

/// Enum to represent agent message output type
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum StepOutputType {
    Info,
    Error,
}

impl fmt::Display for StepOutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            StepOutputType::Info => "I",
            StepOutputType::Error => "E",
        };
        write!(f, "{}", display)
    }
}