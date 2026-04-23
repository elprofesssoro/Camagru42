use crate::controllers;
use crate::headers::{Request, Response, Status};
use crate::middleware::auth::auth_middleware;
use crate::utils::AppState;
use std::sync::Arc;

pub async fn route(request: &mut Request, state: &Arc<AppState>) -> Response {
    if request.content_length < 100 {
        println!("{:?}", request);
    }
	
	match request.method.as_str() {
        "OPTIONS" => Response::empty(Status::Ok),
        "GET" => routing_get(request, state).await,
        "POST" => routing_post(request, state).await,
        "DELETE" => routing_delete(request, state).await,
        _ => Response::empty(Status::BadRequest)
    }
}

async fn routing_get(request: &mut Request, state: &Arc<AppState>) -> Response {
    let route = request.path.strip_prefix("/api/").unwrap_or(&request.path);
	match route {
        "me" => {
            request.user_id = auth_middleware(request, state).await;
            controllers::user::me(request).await
        },
        "verify" => controllers::user::user_verify(request, state).await,
		"re-pass/verify" => controllers::user::re_pass_verify(request, state).await,
        "gallery" => controllers::gallery::gallery(request, state).await,
        "create/history" => {
            request.user_id = auth_middleware(request, state).await;
            controllers::create::create_get(request, state).await
        },
		"create/details" => {
			request.user_id = auth_middleware(request, state).await;
            controllers::create::create_details(request, state).await
		},
		"user/info" => {
			request.user_id = auth_middleware(request, state).await;
			controllers::user::info(request, state).await
		}
        _ => Response::empty(Status::NotFound),
    }
}

async fn routing_post(request: &mut Request, state: &Arc<AppState>) -> Response {
	let route = request.path.strip_prefix("/api/").unwrap_or(&request.path);
	match route {
        "login" => controllers::user::log_in_post(request, state).await,
        "logout" => {
            request.user_id = auth_middleware(request, state).await;
            controllers::user::log_out(request, state).await
        },
        "register" => controllers::user::register(request, state).await,
		"re-email" => controllers::user::re_email(request, state).await,
		"re-pass" => controllers::user::re_pass(request, state).await,
		"re-pass/new" => controllers::user::re_pass_new(request, state).await,
        "gallery/like" => {
            request.user_id = auth_middleware(request, state).await;
            controllers::gallery::like(request, state).await
        }
        "gallery/comment" => {
            request.user_id = auth_middleware(request, state).await;
            controllers::gallery::comment(request, state).await
        }
        "create/post" => {
            request.user_id = auth_middleware(request, state).await;
            controllers::create::create_post(request, state).await
        },
		"user/update" => {
			request.user_id = auth_middleware(request, state).await;
			controllers::user::update(request, state).await
		}
        _ => Response::empty(Status::NotFound),
    }
}

async fn routing_delete(request: &mut Request, state: &Arc<AppState>) -> Response {
    let route = request.path.strip_prefix("/api/").unwrap_or(&request.path);
    match route {
        "create/delete" => {
            request.user_id = auth_middleware(request, state).await;
            controllers::create::create_delete(request, state).await
        },
		"user/delete" => {
			request.user_id = auth_middleware(request, state).await;
			controllers::user::delete(request, state).await
		}
        _ => Response::empty(Status::NotFound),
    }
}
