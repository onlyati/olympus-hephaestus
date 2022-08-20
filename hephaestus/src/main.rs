use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::exit;
use std::os::unix::net::UnixListener;
use std::os::unix::fs::PermissionsExt;
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

mod stream_handler;
mod commands;
mod types;

static HERMES_ADDR: Mutex<Option<String>> = Mutex::new(None);
static VERSION: &str = "v.0.1.2";

fn main() {
    println!("Version {} is starting...", VERSION);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Config file must be specified!");
        exit(1);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Read config file                                                                          */
    /*-------------------------------------------------------------------------------------------*/
    let config = match onlyati_config::read_config(&args[1]) {
        Ok(conf) => conf,
        Err(e) => {
            println!("Failed to parse 'main.conf': {}", e);
            exit(1);
        }
    };

    println!("Configuration:");
    for (setting, value) in &config {
        println!("{} -> {}", setting, value);
    }

    if let Some(addr) = config.get("hermes_addr") {
        let mut hermes_addr = HERMES_ADDR.lock().unwrap();
        *hermes_addr = Some(addr.clone());
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Read argument then check that work directory exist. If it exist set it up work directory  */
    /*-------------------------------------------------------------------------------------------*/
    let work_dir = Path::new(config.get("work_dir").expect("work_dir is not specified in config"));

    if !work_dir.exists() {
        println!("Working directory does not exist: {}", work_dir.display());
        exit(1);
    }

    if let Err(e) = env::set_current_dir(work_dir) {
        println!("Work directory change to {} has failed: {:?}", work_dir.display(), e);
        exit(1);
    }

    println!("Work directory has been found");

    /*-------------------------------------------------------------------------------------------*/
    /* Check work directory structure and fix if possible                                        */
    /* <work_dir>                                                                                */
    /* +-- plans                                                                                 */
    /* '-- logs                                                                                  */
    /*-------------------------------------------------------------------------------------------*/
    let plan_dir = format!("plans");
    let plan_dir = Path::new(&plan_dir);

    if !plan_dir.is_dir() {
        if let Err(e) = fs::create_dir(plan_dir) {
            println!("Failed to create plans directory: {:?}", e);
            exit(1);
        }
    }

    let log_dir = format!("logs");
    let log_dir = Path::new(&log_dir);

    if !log_dir.is_dir() {
        if let Err(e) = fs::create_dir(log_dir) {
            println!("Failed to create logs directory: {:?}", e);
            exit(1);
        }
    }

    println!("Directory check is OK");

    /*-------------------------------------------------------------------------------------------*/
    /* Prepare UNIX socket for listening                                                         */
    /*-------------------------------------------------------------------------------------------*/
    let socket_path = Path::new(config.get("socket_name").expect("socket_name is not specified in config"));

    if socket_path.exists() {
        if let Err(e) = fs::remove_file(socket_path) {
            println!("Error during socket remove: {:?}", e);
            exit(1);
        }
    }

    let listener = match UnixListener::bind(socket_path) {
        Ok(listener) => listener,
        Err(e) => {
            println!("Error during socker preparation: {:?}", e);
            exit(1);
        }
    };

    let mut permission = fs::metadata(socket_path).unwrap().permissions();
    permission.set_mode(0o775);
    if let Err(e) = fs::set_permissions(socket_path, permission) {
        println!("Error during permission change of socket: {:?}", e);
        exit(1);
    }

    let chown = Command::new("/usr/bin/chown")
        .arg("root:olympus")
        .arg(socket_path)
        .output()
        .expect("Ownership change of sockert has failed");

    if !chown.status.success() {
        std::io::stdout().write_all(&chown.stdout).unwrap();
        std::io::stderr().write_all(&chown.stderr).unwrap();
        exit(1);
    }

    println!("Socker '{}' is prepared", socket_path.display());

    /*-------------------------------------------------------------------------------------------*/
    /* Create a list for history and number to track plan IDs. They must be mutexes as they      */
    /* will be handled by threads.                                                               */
    /*-------------------------------------------------------------------------------------------*/
    let history: HashMap<u64, Vec<String>> = HashMap::new();
    let hist_mutex = Arc::new(Mutex::new(history));
    
    /*-------------------------------------------------------------------------------------------*/
    /* It seems everything is okay so far, let's start the listening on socket and see           */
    /* what happens                                                                              */
    /*-------------------------------------------------------------------------------------------*/
    println!("Listeing on '{}' socket", socket_path.display());
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let hist_mutex_clone = Arc::clone(&hist_mutex);
                std::thread::spawn(move || {
                    stream_handler::handle_client(stream, hist_mutex_clone);
                });
            },
            Err(e) => {
                println!("Error occured during streaming: {:?}", e);
            }
        }
    }
}
