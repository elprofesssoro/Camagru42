mod controllers;
mod headers;
mod server;
mod routes;
use server::server;
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    server().await?;
    Ok(())
}
