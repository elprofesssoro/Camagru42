mod controllers;
mod request;
mod server;
mod routes;
use server::server;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    server()?;
    Ok(())
}
