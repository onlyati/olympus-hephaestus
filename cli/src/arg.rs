use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(author, about, long_about)]
pub struct Args {
    /// Specify the action what to do
    #[arg(value_enum)]
    pub action: Action,

    /// Specified plan's name
    #[arg(long)]
    pub plan_name: Option<String>,

    /// Specified plen set's name
    #[arg(long)]
    pub plan_set: Option<String>,

    /// Scheduled plan id
    #[arg(long)]
    pub plan_id: Option<u32>,

    /// Display more information for debugging purpose
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Where it should connect
    /// Allowed formats:
    /// - <protocol>://<hostname>:<port>, for example http://127.0.0.1:3041
    /// - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
    #[arg(short = 'H', long, verbatim_doc_comment, value_parser = check_hostname)]
    pub hostname: String,

    /// Config file for connection details
    #[arg(short, long)]
    pub config: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Action {
    /// List all plan set
    ListPlanSets,

    /// List all plan within a set
    ListPlans,

    /// Get details about specified plan
    ListPlan,

    /// List the scheduled plan output from memory
    Plans,

    /// Show status and log of a scheduled plan
    Status,

    /// Execute a specified plan
    Exec,

    /// Write a specific scheduled plan output into file
    DumpHistory,

    /// Write all scheduled plan output into files
    DumpAllHistory,
}

fn check_hostname(s: &str) -> Result<String, String> {
    if !s.starts_with("http://") && !s.starts_with("https://") && !s.starts_with("cfg://") {
        return Err(String::from("Protocol for hostname can be http:// or https:// or cfg://. "));
    }

    if s.starts_with("http://") || s.starts_with("https://") {
        if !s.contains(':') {
            return Err(String::from("Port number is not specified after the hostname. "));
        }
        else {
            let port = s.split(':').nth(2);
            match port {
                Some(p) => {
                    match p.parse::<u32>() {
                        Ok(num) => {
                            if num > 65535 {
                                return Err(String::from("Port number can be between 0..65535"));
                            }
                        },
                        Err(_) => {
                            return Err(String::from("Failed to convert port number to numbers"));
                        }
                    }
                },
                None => return Err(String::from("Port number is not specified after the hostname. ")),
            }
        }
    }



    return Ok(String::from(s));
}