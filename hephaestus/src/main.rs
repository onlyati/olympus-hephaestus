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

fn main() {
    /*-------------------------------------------------------------------------------------------*/
    /* Read argumen then check that work directory exist. If it exist set it up work directory . */
    /*-------------------------------------------------------------------------------------------*/
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Working directory must be specified!");
        exit(1);
    }

    let work_dir = Path::new(&args[1]);

    if !work_dir.exists() {
        println!("Working directory does not exist: {}", work_dir.display());
        exit(1);
    }

    if let Err(e) = env::set_current_dir(work_dir) {
        println!("Work directory change to {} has failed: {:?}", work_dir.display(), e);
        exit(1);
    }

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

    /*-------------------------------------------------------------------------------------------*/
    /* Prepare UNIX socket for listening                                                         */
    /*-------------------------------------------------------------------------------------------*/
    let socket_path = Path::new("/tmp/hephaestus.sock");

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
