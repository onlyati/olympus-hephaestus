use std::fs;
use std::path::Path;
use std::io::BufRead;
use std::io::BufReader;
use std::thread;

use crate::types::Step;
use crate::types::StepType;
use crate::types::Action;

use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use chrono::Datelike;
use chrono::Local;
use chrono::Timelike;

/// Help command
/// 
/// This command gives and output back about pissible commands
pub fn help(_options: Vec<String>) -> Result<String, String> {
    let mut response = String::new();

    response = response + "Possible actions:\n";
    response = response + "Retrieve list about workflow sets:             list \n";
    response = response + "Retrieve list about workflows within a set:    list <workflow-set>\n";
    response = response + "Retrive details about workflow:                list <workflow-set> <workflow>\n";
    response = response + "Retrieve list about online workflow history:   history\n";
    response = response + "Status of workflow in historical data:         status <workflow-id>\n";
    response = response + "Request to execute a workflow:                 exec <workflow-set> <workflow>\n";

    return Ok(response);
}

/// Execute specifiec step
/// 
/// This function read the file, validate it then execute on same thread as it is
pub fn exec(options: Vec<String>, history: Arc<Mutex<HashMap<u64, Vec<String>>>>) -> Result<String, String> {
    if options.len() < 2 {
        return Err(String::from("Workflow set and workflow also must be specified: exec <workflow-set> <workflow>\n"));
    }
    
    // Read the workflow
    let path = format!("plans/{}/{}.conf", options[0], options[1]);
    let path = Path::new(&path);

    let mut steps = match collect_steps(path) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let mut workflow_index = 0;
    let dt = Local::now();
    let timestamp = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second());
    // Get the next workflow number
    {
        let mut history = history.lock().unwrap();
        for (index, _) in history.iter() {
            if *index >= workflow_index {
                workflow_index = *index + 1;
            }
        }
        let logs: Vec<String> = vec![format!("{} Workflow {} is created", timestamp, workflow_index)];
        history.insert(workflow_index, logs);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Workflow is read, now start to execute its command and act accordingly                    */
    /*-------------------------------------------------------------------------------------------*/
    let copy_hist = Arc::clone(&history);

    let _ = thread::spawn(move || {
        for step in steps.iter_mut() {
            let dt = Local::now();
            let timestamp = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second());
            {
                let mut history = copy_hist.lock().unwrap();
                match history.get_mut(&workflow_index) {
                    Some(v) => {
                        v.push(format!("{} Start to execute step: {}", timestamp, step.step_name));
                    },
                    None => {
                        println!("Internal error occured during creation {}/{}", options[0], options[1]);
                    },
                };
            }

            step.execute();

            let dt = Local::now();
            let timestamp = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second());
            {
                let mut history = copy_hist.lock().unwrap();
                match history.get_mut(&workflow_index) {
                    Some(v) => {
                        v.push(format!("{} Step name: {}, Status: {:?}", timestamp, step.step_name, step.status));
                    },
                    None => {
                        println!("Internal error occured during creation {}/{}", options[0], options[1]);
                    },
                };
            }
        }

        let dt = Local::now();
        let timestamp = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second());
        {
            let mut history = copy_hist.lock().unwrap();
            match history.get_mut(&workflow_index) {
                Some(v) => {
                    v.push(format!("{} Workflow {} is ended", timestamp, workflow_index));
                },
                None => {
                    println!("Internal error occured during creation {}/{}", options[0], options[1]);
                },
            };
        }
    });

    return Ok(format!("Workflow execution has started, ID is: {workflow_index}\n"));
}

/// Get status of the workflow steps
/// 
/// This function return with an output about the status of steps in workflow
pub fn status(options: Vec<String>, history: Arc<Mutex<HashMap<u64, Vec<String>>>>) -> Result<String, String> {
    if options.len() < 1 {
        return Err(String::from("Workflow ID is not specified"));
    }

    let id: u64 = match options[0].parse::<u64>() {
        Err(e) => return Err(format!("Wrong workflow ID is specified: {:?}", e)),
        Ok(v) => v,
    };

    {
        let history = history.lock().unwrap();
        let mut response = String::new();
        match history.get(&id) {
            Some(logs) => {
               for log in logs {
                    response += &log[..];
                    response += "\n";
               }
               return Ok(response);
            },
            None => return Err(format!("No status was found for this ID: {}", id)),
        }
    }
}

/// Read the plan file and create a vector from its steps
/// 
/// This is an internal function in this module. It read and collect information about specified config file.
fn collect_steps(path: &Path) -> Result<Vec<Step>, String> {
    /*-------------------------------------------------------------------------------------------*/
    /* Verify that file does exist                                                               */
    /*-------------------------------------------------------------------------------------------*/
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Error during open '{}': {:?}\n", path.display(), e)),
    };

    let mut collect: bool = false;
    let mut step_raw: String = String::new();

    let mut steps: Vec<Step> = Vec::new();

    /*-------------------------------------------------------------------------------------------*/
    /* Start to read every single line and process them                                          */
    /*-------------------------------------------------------------------------------------------*/
    for line in BufReader::new(file).lines() {
        if let Ok(line_content) = line {
            // If file is empty then nothing to do
            if line_content.is_empty() {
                continue;
            }

            // If file is comment (begins with '#') then nothing to do
            if line_content.len() >= 1 {
                if &line_content[0..1] == "#" {
                    continue;
                }
            }

            // Beginning of a new normal step
            // Start to collect lines into one variable
            if line_content.len() >= 5 {
                if &line_content[0..5] == "<step" {
                    // Step description has begun
                    collect = true;
                }
            }

            // Beginning of a new recovery step
            // Start to collect lines into one variable
            if line_content.len() >= 9 {
                if &line_content[0..9] == "<recovery" {
                    // Step description has begun
                    collect = true;
                }
            }

            /*-----------------------------------------------------------------------------------*/
            /* There was an open tag and we need to collect and process the step                 */
            /*-----------------------------------------------------------------------------------*/
            if collect {
                // Append current data into variable
                step_raw += " ";
                step_raw +=  &line_content[..].trim();

                // If close tag is present, it means we are end of step, start to process collected data
                if line_content.contains("</step>") || line_content.contains("</recovery>") {
                    /*---------------------------------------------------------------------------*/
                    /* Split the line at whitespaces then process every single word              */
                    /*---------------------------------------------------------------------------*/
                    let mut step: Step = Step::new_empty();
                    let mut record_desc: bool = false;               // Description can be more words, must use for tracking its collection
                    let mut record_cmd: bool = false;                // Command can be more words, must use for tracking its collection

                    for word in step_raw.split_whitespace() {
                        // It is a regular step
                        if word == "<step" {
                            step.step_type = StepType::Action;
                            continue;
                        }

                        // It is a recovery step for a regular step
                        if word == "<recovery" {
                            step.step_type = StepType::Recovery;
                            continue;
                        }

                        // Parse the name of the step
                        if word.contains("name=\"") {
                            let parms: Vec<&str> = word.split("\"").collect();
                            if parms.len() < 2 {
                                return Err(format!("Name is not correct, it must be a key-value pair: {:?}", parms));
                            }
                            step.step_name = String::from(parms[1]);
                        }

                        // Parse the parent step and validate that it has been defined earlier
                        if word.contains("parent=\"") {
                            let parms: Vec<&str> = word.split("\"").collect();
                            if parms.len() < 2 {
                                return Err(format!("Description is not correct, it must be a key-value pair: {:?}\n", parms));
                            }
                            
                            for s in steps.iter() {
                                if s.step_name == parms[1] {
                                    step.parent = Some(String::from(parms[1]));
                                    break;
                                }
                            }

                            if let None = step.parent {
                                return Err(format!("Reference as parent for {} but does not exist yet!\n", parms[1]));
                            }
                        }

                        // Start to collect description
                        if word.contains("desc=\"") {
                            let parms: Vec<&str> = word.split("\"").collect();
                            if parms.len() < 2 {
                                return Err(format!("Description is not correct, it must be a key-value pair: {:?}\n", parms));
                            }
                            step.description = String::from(parms[1]);

                            record_desc = true;
                            continue;
                        }

                        if word.contains("\"") & record_desc {
                            let mut pos: usize = 0;
                            for i in word.chars() {
                                if i == '\"' {
                                    break;
                                }
                                pos += 1;
                            }

                            if pos > 0 {
                                step.description += " ";
                                step.description += &word[0..pos];
                            }

                            record_desc = false;
                        }

                        if record_desc {
                            step.description += " ";
                            step.description += word;
                            continue;
                        }

                        // Start to collect command
                        if word.contains(">") {
                            record_cmd = true;
                            continue;
                        }

                        if word == "</recovery>" || word == "</step>" {
                            break;
                        }

                        if record_cmd {
                            match &mut step.action {
                                None => step.action = Some(Action::new(String::from(word))),
                                Some(v) => v.add_arg(String::from(word)),
                            }
                        }
                    }

                    if let Err(e) = step.validate() {
                        return Err(e);
                    }

                    steps.push(step);

                    // Curent step read is ended, reset variables
                    step_raw = String::new();
                    collect = false;
                }
            }
        }
    }

    return Ok(steps);
}

/// List command
/// 
/// This function is called if a list command is received.
/// It is possible to list:
/// - Workflow sets
/// - Workflows within a specified set
/// - Content of a workflow
pub fn list(options: Vec<String>) -> Result<String, String> {
    let mut response = String::new();

    /*-------------------------------------------------------------------------------------------*/
    /* List workflow sets                                                                        */
    /*-------------------------------------------------------------------------------------------*/
    if options.len() == 0 {
        let paths = match fs::read_dir("plans") {
            Ok(paths) => paths,
            Err(e) => return Err(format!("Error during list: {:?}\n", e)),
        };

        for path in paths {
            if let Ok(path) = path {
                let path = path.path();

                if path.is_dir() {
                    let full_path = format!("{}", path.display());
                    match full_path.split("/").collect::<Vec<&str>>().last() {
                        Some(v) => response = response + v + "\n",
                        None => return Err(String::from("Internal error during workflow set scan\n")),
                    }
                }
            }
        }

        return Ok(response);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List workflows in a workflow set                                                          */
    /*-------------------------------------------------------------------------------------------*/
    if options.len() == 1 {
        let paths = match fs::read_dir(format!("plans/{}", options[0])) {
            Ok(paths) => paths,
            Err(e) => return Err(format!("Error during list directory: {:?}", e)),
        };

        for path in paths {
            if let Ok(path) = path {
                let path = path.path();

                if path.is_file() {
                    let full_path = format!("{}", path.display());
                    let full_path: &str = match full_path.split("/").collect::<Vec<&str>>().last() {
                        Some(v) => v,
                        None => return Err(String::from("Internal error during workflow file scan\n")),
                    };

                    let split_path: Vec<&str> = full_path.split(".").collect();
                    
                    if split_path.len() == 0 {
                        return Err(String::from("Internal error during workflow set scan\n"));
                    }

                    if split_path[split_path.len() - 1] != "conf" {
                        continue;
                    }

                    response = response + split_path[0];

                    for i in 1..split_path.len() - 1 {
                        response = response + "." + split_path[i];
                    }
                    
                    response = response + "\n";
                }
            }
        }

        return Ok(response);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Read all workflow file and send it back                                                   */
    /*-------------------------------------------------------------------------------------------*/
    if options.len() == 2 {
        let path = format!("plans/{}/{}.conf", options[0], options[1]);
        let path = Path::new(&path);

        // Let's try to read the specified file
        let steps = match collect_steps(path) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        // If file read was success, then print it into a printable format and send back
        for step in steps {
            let mut line = format!("Name: {}, Type: {:?}, Description: {},", step.step_name, step.step_type, step.description);

            if let Some(parent) = &step.parent {
                line += format!(" Parent step: {},", parent).as_str();
            }

            if let Some(cmd_parm) = &step.action {
                if let Some(cmd) = &cmd_parm.cmd {
                    line += format!(" Command: {}", cmd).as_str();
                }
                for arg in &cmd_parm.args {
                    line += " ";
                    line += &arg[..];
                }
            }

            response += &line[..];
            response += "\n";
        }

        return Ok(response);
    }

    return Ok(String::from("Invalid list parameter"));
}