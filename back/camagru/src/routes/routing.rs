use crate::request::Request;
use crate::controllers;

pub fn route(request: &Request) -> String {
    println!("{:?}", request);
    match request.method.as_str() {
        "GET" => {
            println!("Handling a GET request for path: {}", request.path);
            match routing_get(&request) {
                Some(route) => route,
                None => String::from("Not found"),
            }
        }
        "POST" => {
            println!("Handling a POST request for path: {}", request.path);
            match routing_post(&request) {
                Some(route) => route,
                None => String::from("Not found"),
            }
        }
        _ => {
            println!("Unknown or unsupported method: {}", request.method);
            String::from("Method not supported")
        }
    }
}

fn routing_get(request: &Request) -> Option<String> {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return None;
        }
    };
    let response = match route {
        "login" => Some(controllers::user::log_in(request)),
        _ => None,
    };
    response
}

fn routing_post(request: &Request) -> Option<String> {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return None;
        }
    };

    let response = match route {
        "login" => Some(controllers::user::log_in(request)),
        "register" => Some(controllers::user::register(request)),
        _ => None,
    };
    response
}
