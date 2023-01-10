use std::fs;
use std::path::Path;
use std::io::{BufReader, BufRead};

use crate::structs::plan::Plan;
use crate::structs::step::Step;
use crate::structs::action::Action;
use crate::structs::enums::StepType;

/// Read the plan file and create a vector from its steps
/// 
/// This is an internal function in this module. It read and collect information about specified config file.
pub fn collect_steps(path: &Path) -> Result<Plan, String> {
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
        let mut cwd = None;

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
                    let mut record_env: bool = false;
                    let mut key_env = String::new();
                    let mut value_env = String::new();

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

                        if word.contains("user=\"") {
                            let parms: Vec<&str> = word.split("\"").collect();
                            if parms.len() < 2 {
                                return Err(format!("User is not correct, it must be a key-value pair: {:?}", parms));
                            }
                            step.user = Some(String::from(parms[1]));
                        }

                        if word.contains("cwd=\"") {
                            let parms: Vec<&str> = word.split("\"").collect();
                            if parms.len() < 2 {
                                return Err(format!("Work directory is not correct, it must be a key-value pair: {:?}", parms));
                            }
                            cwd = Some(String::from(parms[1]));
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
                                return Err(format!("Parent is not correct, it must be a key-value pair: {:?}\n", parms));
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

                        // Start to collect environment variables
                        if word.contains("setenv=\"") {
                            let parms: Vec<&str> = word.split("\"").collect();
                            if parms.len() > 1 {
                                   key_env = String::from(parms[1]);
                            }
                            record_env = true;
                            continue;
                        }

                        if word.contains("\"") & record_env {
                            let mut pos: usize = 0;
                            for i in word.chars() {
                                if i == '\"' {
                                    break;
                                }
                                pos += 1;
                            }
                            value_env += &word[0..pos];

                            if value_env.is_empty() || key_env.is_empty() {
                                return Err(String::from("Key and/or value is missing in setenv option"));
                            }

                            step.envvars.insert(key_env.clone(), value_env.clone());
                            value_env = String::new();
                            key_env = String::new();
                            record_env = false;
                        }

                        if record_env {
                            if key_env.is_empty() {
                                key_env = String::from(word);
                            } else {
                                value_env += word;
                                value_env += " ";
                            }
                            continue;
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
                                None => step.action = Some(Action::new(String::from(word), cwd.clone())),
                                Some(v) => v.cmd.push(String::from(word)),
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