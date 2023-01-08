use std::process::exit;

use clap::Parser;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use hephaestus::hephaestus_client::HephaestusClient;
use hephaestus::{Empty, List, PlanSetArg, PlanArg, PlanId, Dictionary, PlanStep, PlanDetails, PlanHistory, PlanList};

mod hephaestus {
    tonic::include_proto!("hephaestus");
}


mod arg;
use arg::{Args, Action};

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        main_async().await.unwrap();
    });
}

async fn main_async() -> Result<i32, Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Measure runtime of script
    let start = std::time::Instant::now();

    // Try to connect to gRPC server
    let grpc_channel = Channel::from_shared(args.hostname.clone())
        .unwrap()
        .connect()
        .await?;

    let mut grpc_client = HephaestusClient::new(grpc_channel);

    let mut final_rc = 0;

    match args.action {
        Action::ListPlanSets => {
            let response: Result<Response<List>, Status> = grpc_client.list_plan_sets(Request::new(Empty {})).await;

            match response {
                Ok(resp) => {
                    let list = resp.into_inner();
                    let mut list = list.list;
                    list.sort();

                    for set in list {
                        println!("{}", set);
                    }
                },
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        },
        Action::ListPlans => {
            if args.plan_set.is_some() {
                let params = PlanSetArg {
                    name: args.plan_set.clone().unwrap(),
                };
                let response: Result<Response<List>, Status> = grpc_client.list_plans(Request::new(params)).await;

                match response {
                    Ok(resp) => {
                        let list = resp.into_inner();
                        let mut list = list.list;
                        list.sort();
    
                        for set in list {
                            println!("{}", set);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed request: {}", e.message());
                        final_rc = 4;
                    }
                }
            }
            else {
                eprintln!("Plan set parameter is not set");
                final_rc = 2;
            }
        },
        Action::ListPlan => {
            if args.plan_set.is_some() && args.plan_name.is_some() {
                let params = PlanArg {
                    set: args.plan_set.clone().unwrap(),
                    plan: args.plan_name.clone().unwrap(),
                };
                let response: Result<Response<PlanDetails>, Status> = grpc_client.list_plan(params).await;

                match response {
                    Ok(resp) => {
                        let plan = resp.into_inner();

                        println!("Details about {}/{} plan:", args.plan_set.clone().unwrap(), plan.id);

                        for step in plan.steps {
                            println!("{} - {}", step.name, step.desc);
                            println!("- Type:                  {}", step.r#type);

                            if !step.user.is_empty() {
                                println!("- Assigned user:       {}", step.user);
                            }
                            
                            println!("- Command:               {}", step.action);

                            if !step.parent.is_empty() {
                                println!("- Depend from:         {}", step.parent);
                            }

                            if step.envvars.len() > 0 {
                                println!("- Environment variables:");
                                for elem in step.envvars {
                                    println!("   - {} = {}", elem.key, elem.value);
                                }
                            }

                            println!("");
                        }

                    },
                    Err(e) => {
                        eprintln!("Failed request: {}", e.message());
                        final_rc = 4;
                    },
                }
            }
            else {
                eprintln!("Plan set and plan name have to be set");
                final_rc = 2;
            }
        },
        Action::Plans => {
            let response: Result<Response<PlanList>, Status> = grpc_client.show_plans(Request::new(Empty { })).await;

            match response {
                Ok(resp) => {
                    let plans = resp.into_inner();
                    let mut plans = plans.ids;
                    plans.sort_by(|a, b| a.id.cmp(&b.id));

                    for plan_id in plans {
                        println!("{}", plan_id.id);
                    }
                },
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        },
        Action::Exec => {
            if args.plan_set.is_some() && args.plan_name.is_some() {
                let params = PlanArg {
                    set: args.plan_set.clone().unwrap(),
                    plan: args.plan_name.clone().unwrap(),
                };
                let response: Result<Response<PlanId>, Status> = grpc_client.execute(params).await;

                match response {
                    Ok(resp) => {
                        let plan_id = resp.into_inner();

                        println!("Batch is started, id: {}", plan_id.id);
                    }
                    Err(e) => {
                        eprintln!("Failed request: {}", e.message());
                        final_rc = 4;
                    }
                }
            }
            else {
                eprintln!("Plan set and plan name have to be set");
                final_rc = 2;
            }
        },
        Action::Status => {
            match args.plan_id {
                Some(id) => {
                    let params = PlanId {
                        id: id,
                    };
                    let response: Result<Response<PlanHistory>, Status> = grpc_client.show_status(params).await;

                    match response {
                        Ok(resp) => {
                            let hist = resp.into_inner();

                            for line in hist.history {
                                println!("{}", line);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed request: {}", e.message());
                            final_rc = 4;
                        }
                    }
                },
                None => {
                    eprintln!("Plan id have to be set");
                    final_rc = 2;
                }
            }
        },
        Action::DumpAllHistory => {

        },
        Action::DumpHistory => {

        },
    }

    let elapsed = start.elapsed();
    print_verbose(&args, format!("Measured runtime: {:?}", elapsed));

    return Ok(final_rc);
}

fn print_verbose<T: std::fmt::Display>(args: &Args, text: T) {
    if args.verbose {
        println!("> {}", text);
    }
}
