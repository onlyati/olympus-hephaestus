use std::fmt;

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

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut full_cmd = self.cmd.join(" ");
        if let Some(cwd) = &self.cwd {
            full_cmd = format!("cd {} && {}", cwd, full_cmd);
        }
        write!(f, "{}", full_cmd)
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