use std::collections::HashMap;

use tonic::transport::{Channel, Certificate, ClientTlsConfig};
use tonic::{Request, Response, Status};

use hermes::hermes_client::{HermesClient};
use hermes::{SetPair, Pair};

mod hermes {
    tonic::include_proto!("hermes");
}

pub async fn start_hermes_client(config: &HashMap<String, String>, receiver: &mut tokio::sync::mpsc::Receiver<(String, String)>) -> Result<(), Box<dyn std::error::Error>> {

    let addr = config.get("hermes.grpc.address").unwrap();
    let table = config.get("hermes.table").unwrap();

    let tls = match config.get("hermes.grpc.tls") {
        Some(tls) => tls.clone(),
        None => String::from("no"),
    };
    let tls_cert = config.get("hermes.grpc.tls.ca_cert");
    let tls_domain = config.get("hermes.grpc.tls.domain");

    let channel = if tls == "yes" && tls_cert.is_some() && tls_domain.is_some() {
        let pem = tokio::fs::read(tls_cert.unwrap()).await.unwrap();
        let ca = Certificate::from_pem(pem);

        let tls = ClientTlsConfig::new()
            .ca_certificate(ca)
            .domain_name(tls_domain.unwrap());

        Channel::from_shared(addr.clone())
            .unwrap()
            .tls_config(tls)
            .unwrap()
            .connect()
            .await
            .unwrap()
    }
    else {
        Channel::from_shared(addr.clone())
            .unwrap()
            .connect()
            .await
            .unwrap()
    };

    let mut client = HermesClient::new(channel);
    println!("Hermes client is ready");

    while let Some(message) = receiver.recv().await {
        println!("Update Hermes with {:?}", message);
        let pair = SetPair {
            key: message.0,
            table: table.clone(),
            value: message.1,
        };

        let request = Request::new(pair);
        let response: Result<Response<Pair>, Status> = client.set(request).await;

        if let Err(e) = response {
            eprintln!("Failed to update Hermes: {}", e.message());
        }
    }

    return Ok(());
}