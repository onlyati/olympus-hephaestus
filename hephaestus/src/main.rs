use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::sync::Mutex;
use std::sync::RwLock;
use std::collections::HashMap;

use tokio::sync::{mpsc::Sender, mpsc::Receiver};

mod services;
mod structs;

static GLOBAL_CONFIG: RwLock<Option<HashMap<String, String>>> = RwLock::new(None);
static HISTORY: RwLock<Option<HashMap<structs::historey_key::HistoryKey, Vec<String>>>> = RwLock::new(None);
static HERMES_RX: Mutex<Option<Receiver<(String, String)>>> = Mutex::new(None);
static HERMES_TX: Mutex<Option<Sender<(String, String)>>> = Mutex::new(None);
static VERSION: &str = "v.0.2.0";

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
            println!("Failed to parse '{}': {}", args[1], e);
            exit(1);
        }
    };

    println!("Configuration:");
    for (setting, value) in &config {
        println!("{} -> {}", setting, value);
    }

    {
        let mut glob_config = GLOBAL_CONFIG.write().unwrap();
        *glob_config = Some(config.clone());
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Check work directory structure and fix if possible                                        */
    /* <work_dir>                                                                                */
    /* +-- plans                                                                                 */
    /* '-- logs                                                                                  */
    /*-------------------------------------------------------------------------------------------*/
    if let Some(plan_dir) = config.get("plan.rule_dir") {
        let plan_dir = Path::new(&plan_dir);

        if !plan_dir.is_dir() {
            if let Err(e) = fs::create_dir(plan_dir) {
                println!("Failed to create plans directory: {:?}", e);
                exit(1);
            }
        }
    }

    if let Some(log_dir) = config.get("plan.rule_log") {
        let log_dir = Path::new(&log_dir);

        if !log_dir.is_dir() {
            if let Err(e) = fs::create_dir(log_dir) {
                println!("Failed to create logs directory: {:?}", e);
                exit(1);
            }
        }
    }

    println!("Directory check is OK");

    /*-------------------------------------------------------------------------------------------*/
    /* Initailize history                                                                        */
    /*-------------------------------------------------------------------------------------------*/
    {
        let mut history = HISTORY.write().unwrap();
        *history = Some(HashMap::new());
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Allocate a tokio runtime and start Hermes client if required                              */
    /*-------------------------------------------------------------------------------------------*/
    if let Some(ena) = config.get("hermes.enable") {
        if ena == "yes" && config.get("hermes.grpc.address").is_some() && config.get("hermes.table").is_some() {
            println!("Corresponse properties are set to yes, so start Hermes client");
            let config2 = config.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let rt = match rt {
                    Ok(rt) => rt,
                    Err(e) => panic!("Failed to allocated runtime for Hermes client: {}", e),
                };

                let (tx, rx) = tokio::sync::mpsc::channel(256);
                {
                    let mut grx = HERMES_RX.lock().unwrap();
                    *grx = Some(rx);

                    let mut grt = HERMES_TX.lock().unwrap();
                    *grt = Some(tx);
                }                

                rt.block_on(async move {
                    loop {
                        let _ = services::hermes_client::start_hermes_client(&config2).await;
                        eprintln!("Hermes client has failed, try to restart 30 sec later");
                        tokio::time::sleep(tokio::time::Duration::new(30, 0)).await;
                    }
                })
            });
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Allocate a tokio runtime, then start gRPC server                                          */
    /*-------------------------------------------------------------------------------------------*/
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    
    rt.block_on(async move {
        services::grpc::start_server(&config).await.expect("Failed to start gRPC server");
    });
    
}
