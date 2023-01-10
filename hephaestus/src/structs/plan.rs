use super::enums::StepStatus;
use super::step::Step;

/// A plan consist of more step which can depend from each other
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