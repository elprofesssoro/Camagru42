mod controllers;
mod dto;
mod middleware;
mod repositories;
mod server;
mod utils;
use server::server;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    server().await?;
    Ok(())
}
