use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::os::unix::net::UnixListener;

mod stream_handler;

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
    let plan_dir = format!("{}/plans", args[1]);
    let plan_dir = Path::new(&plan_dir);

    if !plan_dir.is_dir() {
        if let Err(e) = fs::create_dir(plan_dir) {
            println!("Failed to create plans directory: {:?}", e);
            exit(1);
        }
    }

    let log_dir = format!("{}/logs", args[1]);
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

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    stream_handler::handle_client(stream);
                });
            },
            Err(e) => {
                println!("Error occured during streaming: {:?}", e);
            }
        }
    }
}
