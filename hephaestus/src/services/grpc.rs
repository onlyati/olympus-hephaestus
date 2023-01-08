use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::str::FromStr;
use tonic::{transport::Server, Request, Response, Status};

use hephaestus::hephaestus_server::{Hephaestus, HephaestusServer};
use hephaestus::{Empty, List, PlanSetArg, PlanArg, PlanId, Dictionary, PlanStep, PlanDetails, PlanHistory};

use crate::GLOBAL_CONFIG;

mod hephaestus {
    tonic::include_proto!("hephaestus");
}

#[derive(Debug, Default)]
struct HephaestusGrpc {}

#[tonic::async_trait]
impl Hephaestus for HephaestusGrpc {
    /// This gRPC endpoint list all directory within plan.rule_dir
    async fn list_plan_sets(&self, _request: Request<Empty>) -> Result<Response<List>, Status> {
        let config = GLOBAL_CONFIG.read().unwrap();
        let config = match &*config {
            Some(config) => config,
            None => return Err(Status::internal(String::from("Configuration is not available"))),
        };

        let mut sets: Vec<String> = Vec::new();

        match config.get("plan.rule_dir") {
            Some(rule_dir) => {
                let paths = match fs::read_dir(rule_dir) {
                    Ok(p) => p,
                    Err(e) => return Err(Status::internal(format!("Couldn't read from '{}': {}", rule_dir, e))),
                };

                for path in paths {
                    if let Ok(path) = path {
                        let path = path.path();

                        if path.is_dir() {
                            let full_path = format!("{}", path.display());
                            match full_path.split("/").collect::<Vec<&str>>().last() {
                                Some(v) => sets.push(v.to_string()),
                                None => return Err(Status::internal(String::from("Could not parse directory"))),
                            }
                        }
                    }
                }
            }
            None => return Err(Status::internal(String::from("Property 'plan.rule_dir' is not specified in config"))),
        }

        let list = List {
            list: sets,
        };

        return Ok(Response::new(list));
    }

    /// This gRPC endpoint list all *.conf files within specified plan set
    async fn list_plans(&self, request: Request<PlanSetArg>) -> Result<Response<List>, Status> {
        let arg = request.into_inner();
        let set_name = arg.name;

        let config = GLOBAL_CONFIG.read().unwrap();
        let config = match &*config {
            Some(config) => config,
            None => return Err(Status::internal(String::from("Configuration is not available"))),
        };

        let mut rules: Vec<String> = Vec::new();

        match config.get("plan.rule_dir") {
            Some(rule_dir) => {
                let paths = match fs::read_dir(format!("{}/{}", rule_dir, set_name)) {
                    Ok(p) => p,
                    Err(e) => return Err(Status::internal(format!("Couldn't read from '{}': {}", rule_dir, e))),
                };

                for path in paths {
                    if let Ok(path) = path {
                        let path = path.path();

                        if path.is_file() {
                            let full_path = format!("{}", path.display());
                            let full_path: &str = match full_path.split("/").collect::<Vec<&str>>().last() {
                                Some(v) => v,
                                None => return Err(Status::internal(String::from("Could not parse directory"))),
                            };

                            let rule_name: String = full_path.ends_with(".conf").to_string();
                            rules.push(rule_name);                            
                        }
                    }
                }
            }
            None => return Err(Status::internal(String::from("Property 'plan.rule_dir' is not specified in config"))),
        }

        let list = List {
            list: rules,
        };

        return Ok(Response::new(list));
    }

    /// This gRPC endpoint returns with details of a specified rule
    async fn list_plan(&self, request: Request<PlanArg>) -> Result<Response<PlanDetails>, Status> {
        let arg = request.into_inner();
        let set = arg.set;
        let plan_name = arg.plan;

        let config = GLOBAL_CONFIG.read().unwrap();
        let config = match &*config {
            Some(config) => config,
            None => return Err(Status::internal(String::from("Configuration is not available"))),
        };

        let rule_dir = match config.get("plan.rule_dir") {
            Some(rule_dir) => rule_dir,
            None => return Err(Status::internal(String::from("Property 'plan.rule_dir' is not specified in config"))),
        };

        let rule_path = format!("{}/{}/{}.conf", rule_dir, set, plan_name);
        let rule_path = Path::new(&rule_path);

        if !rule_path.exists() {
            return Err(Status::not_found(String::from("Specified rule does not exist")));
        }

        let plan = match super::parser::collect_steps(rule_path) {
            Ok(plan) => plan,
            Err(e) => return Err(Status::internal(format!("Failed to parse rule: {}", e))),
        };

        let steps: Vec<PlanStep> = plan.steps.iter()
            .map(|x| PlanStep {
                name: x.step_name.clone(),
                desc: x.description.clone(),
                r#type: format!("{:?}", x.step_type),
                user: if x.user.is_some() { x.user.clone().unwrap() } else { String::new() },
                action: format!("{}", x.action.clone().unwrap()),
                parent: if x.parent.is_some() { x.parent.clone().unwrap() } else { String::new() },
                envvars: {
                    let mut vars: Vec<Dictionary> = Vec::new();

                    for (key, value) in &x.envvars {
                        vars.push(Dictionary {
                            value: value.clone(),
                            key: key.clone(),
                        });
                    }

                    vars
                },
            })
            .collect();
        
        let plan = PlanDetails {
            id: plan_name,
            steps: steps,
        };

        return Ok(Response::new(plan));
    }

    async fn show_hist(&self, request: Request<Empty>) -> Result<Response<List>, Status> {
        unimplemented!();
    }

    async fn show_status(&self, request: Request<PlanId>) -> Result<Response<PlanHistory>, Status> {
        unimplemented!();
    }

    async fn execute(&self, request: Request<PlanArg>) -> Result<Response<Empty>, Status> {
        unimplemented!();
    }

    async fn dump_hist(&self, request: Request<PlanId>) -> Result<Response<Empty>, Status> {
        unimplemented!();
    }

    async fn dump_hist_all(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        unimplemented!();
    }
}

pub async fn start_server(config: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    match config.get("host.grpc.address") {
        Some(addr) => {
            let hepha_grpc = HephaestusGrpc::default();
            let hepha_service = HephaestusServer::new(hepha_grpc);

            let addr_list = tokio::net::lookup_host(addr).await?;

            let mut addr: Option<String> = None;
            for a in addr_list {
                addr = Some(format!("{}", a));
            }
            let addr = addr.unwrap();
            let addr = std::net::SocketAddr::from_str(&addr[..])?;

            println!("Start gRPC endpoint on {}", addr);
            Server::builder()
                .add_service(hepha_service)
                .serve(addr)
                .await?;
        }
        None => eprintln!("Hostname and port is not found in config with 'host.grpc.address' property"),
    }

    return Ok(());
}