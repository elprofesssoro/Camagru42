mod controllers;
mod headers;
mod utils;
mod server;
mod routes;
mod dto;
mod middleware;
mod repositories;
use server::server;
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    server().await?;
    Ok(())
}
