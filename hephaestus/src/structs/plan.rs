use super::enums::StepStatus;
use super::step::Step;

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