mod server;
use tonic::{transport::Server, Request, Response, Status};
use crate::server::ConcreteEmailSender;
use crate::server::EmailSenderServer;
use std::env;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::from_path(".env").expect("dot env error");
    let addr = "0.0.0.0:5000".parse()?;
    let sender = ConcreteEmailSender::default();
    println!("started email sender server");
    Server::builder()
        .add_service(EmailSenderServer::new(sender))
        .serve(addr)
        .await?;

    Ok(())
}