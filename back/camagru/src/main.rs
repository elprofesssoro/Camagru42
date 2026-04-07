mod controllers;
mod headers;
mod server;
mod routes;
mod dto;
mod middleware;
use server::server;
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    server().await?;
    Ok(())
}
