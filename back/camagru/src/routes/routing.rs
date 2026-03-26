use crate::headers::{Request, Response, Status};
use crate::controllers;

pub async fn route(request: &Request) -> Response {
    println!("{:?}", request);
	match request.method.as_str() {
		"OPTIONS" => {
			Response::empty(Status::Ok)
		},
        "GET" => {
            println!("Handling a GET request for path: {}", request.path);
          	routing_get(&request)
        }
        "POST" => {
            println!("Handling a POST request for path: {}", request.path);
            routing_post(&request)
        }
        _ => {
            println!("Unknown or unsupported method: {}", request.method);
			Response::empty(Status::BadRequest)
        }
    }
}

fn routing_get(request: &Request) -> Response {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return Response::empty(Status::NotFound);
        }
    };
    let response = match route {
        "login" => {
			controllers::user::log_in_get(request)
		},
        _ => Response::empty(Status::NotFound),
    };
    response
}

fn routing_post(request: &Request) -> Response {
    let route = match request.path.strip_prefix("/api/") {
        Some(route) => route,
        None => {
            return Response::empty(Status::NotFound);
        }
    };

    let response = match route {
        "login" => {
			controllers::user::log_in_post(request)
		},
        "register" => {
			controllers::user::register(request)
		},
        _ =>  {
			Response::empty(Status::NotFound)
		},
    };
    response
}
