use crate::request::Request;

pub fn log_in(request: &Request) -> String{
    println!("Logging in...");
    String::from("Logged in!")
}

pub fn register(request: &Request) -> String{
    println!("Registering...");
    let body_str = match &request.body {
        Some(body) => String::from_utf8_lossy(body).to_string(),
        None => String::new(),
    };
    println!("Received body: {}", body_str);
    String::from("Registered!")
}