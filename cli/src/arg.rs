use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, about, long_about)]
pub struct Args {
    /// Specify the action what to do
    #[command(subcommand)]
    pub action: Action,

    /// Where it should connect
    /// Allowed formats:
    /// - <protocol>://<hostname>:<port>, for example http://127.0.0.1:3041
    /// - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
    #[arg(short = 'H', long, verbatim_doc_comment, value_parser = check_hostname)]
    pub hostname: String,

    /// Config file for connection details
    #[arg(short, long, default_value_t = String::from("/etc/olympus/hephaestus/client.conf"))]
    pub config: String,

    /// Show more detail about connection
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// List all plan set
    ListPlanSets,

    /// List all plan within a set
    ListPlans {
        /// Specified plen set's name
        #[arg(long)]
        set: String,
    },

    /// Get details about specified plan
    ListPlan {
        /// Specified plan's name
        #[arg(long)]
        name: String,

        /// Specified plen set's name
        #[arg(long)]
        set: String,
    },

    /// List the scheduled plan output from memory
    Plans,

    /// Show status and log of a scheduled plan
    Status {
        /// Scheduled plan id
        #[arg(long)]
        id: u32,
    },

    /// Execute a specified plan
    Exec {
        /// Specified plan's name
        #[arg(long)]
        name: String,

        /// Specified plen set's name
        #[arg(long)]
        set: String,
    },

    /// Write a specific scheduled plan output into file
    DumpHistory {
        /// Scheduled plan id
        #[arg(long)]
        id: u32,
    },

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