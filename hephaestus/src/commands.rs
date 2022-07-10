use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::thread;
use std::mem::size_of;

use crate::types::Plan;
use crate::types::Step;
use crate::types::StepType;
use crate::types::Action;
use crate::types::StepStatus;

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
    response = response + "List workflow IDs:                             workflows\n";
    response = response + "Status of workflow in historical data:         status <workflow-id>\n";
    response = response + "Request to execute a workflow:                 exec <workflow-set> <workflow>\n";
    response = response + "Dump log from memory:                          dump log\n";

    return Ok(response);
}

/// Execute specifiec step
/// 
/// This function read the file, validate it then execute on same thread as it is
pub fn exec(options: Vec<String>, history: Arc<Mutex<HashMap<u64, Vec<String>>>>) -> Result<String, String> {
    if options.len() < 2 {
        return Err(String::from("Workflow set and workflow also must be specified: exec <workflow-set> <workflow>\n"));
    }
    
    /*-------------------------------------------------------------------------------------------*/
    /* Read and verify workflow file                                                             */
    /*-------------------------------------------------------------------------------------------*/
    let mut workflow_index = 0;

    // Get the next workflow number
    {
        let history = history.lock().unwrap();
        for (index, _) in history.iter() {
            if *index >= workflow_index {
                workflow_index = *index + 1;
            }
        }
    }

    let path = format!("plans/{}/{}.conf", options[0], options[1]);
    let path = Path::new(&path);

    let mut plan = match collect_steps(path) {
        Ok(v) => v,
        Err(e) => {
            write_history(format!("Workflow initialization has failed: {}", e), &String::from("ERROR"), workflow_index, &history);
            return Err(e);
        },
    };

    write_history(String::from("Workflow is created"), &plan.id, workflow_index, &history);

    /*-------------------------------------------------------------------------------------------*/
    /* Workflow is read, now start to execute its command and act accordingly                    */
    /*-------------------------------------------------------------------------------------------*/
    let copy_hist = Arc::clone(&history);

    let _ = thread::spawn(move || {
        let mut completion_list: HashMap<&String, Step> = HashMap::new();  // We will save the previous steps for parent checking

        /*---------------------------------------------------------------------------------------*/
        /* Run through the step list. Step will be executed if:                                  */
        /* 1. This is a regular step and has no parent or parent run OK                          */
        /* 2. This is a recovery step and its parent step has Failed or NOK                      */
        /* Any other case, step remains NoRun status                                             */
        /*---------------------------------------------------------------------------------------*/
        for step in plan.steps.iter_mut() {
            write_history(format!("{} => Pending", step.step_name), &plan.id, workflow_index, &copy_hist);

            let mut enable = false;

            match &step.parent {
                Some(p) => {
                    if let Some(v) = completion_list.get(p) {
                        if (v.status == StepStatus::Ok && step.step_type == StepType::Action) || 
                           ((v.status == StepStatus::Failed || v.status == StepStatus::Nok) && step.step_type == StepType::Recovery) {
                            enable = true;
                        }
                    }
                },
                None => {
                    enable = true;
                }
            }

            if enable {
                match step.execute() {
                    Some(log) => {
                        for line in log.lines() {
                            write_history(String::from(line), &plan.id, workflow_index, &copy_hist);
                        }
                    }
                    None => (),
                }
                completion_list.insert(&step.step_name, step.clone());
            }

            write_history(format!("{} => {:?}", step.step_name, step.status), &plan.id, workflow_index, &copy_hist);
        }

        write_history(String::from("Workflow is ended"), &plan.id, workflow_index, &copy_hist);
    });

    /*-------------------------------------------------------------------------------------------*/
    /* Execution started asyncronically, return to user that it has been started                 */
    /*-------------------------------------------------------------------------------------------*/
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
fn collect_steps(path: &Path) -> Result<Plan, String> {
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
    let mut plan_id: (bool, String) = (false, String::new());

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

            // Beginning of a plan descriptor
            if line_content.len() >= 5 {
                if &line_content[0..5] == "<plan" {
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

                // Porcess plan tag
                if line_content.contains("</plan>") {
                    for word in step_raw.split_whitespace() {
                        // It is the plan descriptor
                        if word == "<plan" {
                            plan_id.0 = true;
                            continue;
                        }

                        if plan_id.0 && word.contains("id=\"") {
                            let parms: Vec<&str> = word.split("\"").collect();
                            if parms.len() < 2 {
                                return Err(format!("Name is not correct, it must be a key-value pair: {:?}", parms));
                            }
                            plan_id.1 = String::from(parms[1]);
                        }

                        if word.contains("</plan>") && plan_id.0 {
                            plan_id.0 = false;
                        }
                    }

                    step_raw = String::new();
                    collect = false;
                }

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

    if plan_id.1.is_empty() {
        return Err(String::from("Plan ID is missing"));
    }

    return Ok(Plan::new(plan_id.1, steps));
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
        let plan = match collect_steps(path) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}/{}: {}", std::env::current_dir().unwrap().display(), path.display(), e)),
        };

        response += "Plan ID: ";
        response += &plan.id[..];
        response += "\n";

        // If file read was success, then print it into a printable format and send back
        for step in plan.steps {
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

/// List all IDs
/// 
/// List all IDs which is in memory of process
pub fn list_ids(_options: Vec<String>, history: Arc<Mutex<HashMap<u64, Vec<String>>>>) -> Result<String, String> {
    let mut response = String::new();

    {
        let history = history.lock().unwrap();
        for (index, _) in history.iter() {
            response += &format!("{}\n", index);
        }
    }

    return Ok(response);
}

/// Dump
/// 
/// Dump log from memory onto file
pub fn dump(options: Vec<String>, history: Arc<Mutex<HashMap<u64, Vec<String>>>>) -> Result<String, String> {
    if options.len() < 1 {
        return Err(String::from("Missing parameter for dump"));
    }

    if options[0] != String::from("log") {
        return Err(String::from("Invalid dump option"));
    }

    {
        let mut history = history.lock().unwrap();
        
        let mut keys: Vec<(u64, String)> = Vec::with_capacity(history.len() * (size_of::<u64>() + size_of::<String>()));

        for (index, logs) in history.iter() {
            if logs.len() > 0 {
                let words: Vec<&str> = logs[0].split_whitespace().collect();
                let file_name = words[2].replace("(", "_");
                let file_name = file_name.replace(")", "");
                let path = format!("logs/{}_{}_{}.log", file_name, words[0], words[1]);

                keys.push((*index, path));
            }
        }

        for key in &keys {
            if let Some(logs) = history.get(&key.0) {
                let path = Path::new(&key.1);
                let mut log_file = File::create(path).unwrap();

                for log in logs {
                    writeln!(log_file, "{}", log).unwrap();
                }
            }
        }

        for key in keys {
            history.remove(&key.0).unwrap();
        }
    }

    return Ok(String::from("OK"));
}

/// Function to write into history hashmap
fn write_history(text: String, plan_name: &String, index: u64, history: &Arc<Mutex<HashMap<u64, Vec<String>>>>) {
    let dt = Local::now();
    let timestamp = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second());
    
    let job = format!("{}({})", plan_name, index);
    let text = format!("{} {:32} {}", timestamp, job, text);

    let mut history = history.lock().unwrap();
    match history.get_mut(&index) {
        Some(v) => {
            v.push(text);
        },
        None => {
            // Not ideal but it can happen that dump log has run while workflow was executed and hashmap has been erased
            // It needs to create the key again
            let msg: Vec<String> = vec![text];
            history.insert(index, msg);
        },
    };
}