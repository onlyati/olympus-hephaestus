use std::fs;
use std::path::Path;
use std::io::BufRead;
use std::io::BufReader;

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

pub fn list(options: Vec<String>) -> Result<String, String> {
    let mut response = String::new();

    /*-------------------------------------------------------------------------------------------*/
    /* List workflow sets                                                                        */
    /*-------------------------------------------------------------------------------------------*/
    if options.len() == 0 {
        let paths = match fs::read_dir("plans") {
            Ok(paths) => paths,
            Err(e) => return Err(format!("Error during list: {:?}", e)),
        };

        for path in paths {
            if let Ok(path) = path {
                let path = path.path();

                if path.is_dir() {
                    let full_path = format!("{}", path.display());
                    match full_path.split("/").collect::<Vec<&str>>().last() {
                        Some(v) => response = response + v + "\n",
                        None => return Err(String::from("Internal error during workflow set scan")),
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
                        None => return Err(String::from("Internal error during workflow file sca")),
                    };

                    let split_path: Vec<&str> = full_path.split(".").collect();
                    
                    if split_path.len() == 0 {
                        return Err(String::from("Internal error during workflow set scan"));
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

        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Error during open '{}': {:?}", path.display(), e)),
        };

        for line in BufReader::new(file).lines() {
            if let Ok(line_content) = line {
                response = response + &line_content[..] + "\n";
            }
        }

        return Ok(response);
    }

    return Ok(String::from("Invalid list parameter"));
}