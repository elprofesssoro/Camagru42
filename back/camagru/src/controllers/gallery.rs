use crate::dto::gallery_dto::{CommentDTO, PaginatedGalleryDTO};
use crate::headers::{Request, Response, Status};
use crate::unwrap_or_return;
use crate::repositories::gallery_repo::{GalleryRepo, NotificationData};
use crate::utils::{extract_json, log_error, send_email, AppState, EmailConfig};
use std::sync::Arc;

pub async fn gallery(request: &Request, state: &Arc<AppState>) -> Response {
    let query = unwrap_or_return!(request.query.as_ref(), Status::BadRequest);
    let (page, per_page) = unwrap_or_return!(validate_gallery_query(query), Status::BadRequest);

    println!("Page: {}, Per Page: {}", page, per_page);

    let safe_per_page = if per_page > 50 { 50 } else { per_page } as i64;
    let current_page = if page == 0 { 1 } else { page } as i64;
    let offset = (current_page - 1) * safe_per_page;

    match GalleryRepo::get_paginated_posts(&state.db, safe_per_page, offset).await {
        Ok((total_posts, posts)) => {
            let response_data = PaginatedGalleryDTO {
                posts,
                total_posts: total_posts as usize,
            };

            match serde_json::to_string(&response_data) {
                Ok(json) => Response::json(json),
                Err(e) => {
                    log_error("Error in PaginatedGallery serialization", e);
                    Response::empty(Status::InternalServerError)
                }
            }
        }
        Err(e) => {
            log_error("Database error fetching paginated posts", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn like(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let query = unwrap_or_return!(request.query.as_ref(), Status::BadRequest);
    let post_id = unwrap_or_return!(validate_like_query(query), Status::BadRequest);

    match GalleryRepo::toggle_like(&state.db, user_id, post_id).await {
        Ok(false) => Response::empty(Status::Ok),
        Ok(true) => Response::empty(Status::Created),
        Err(err) => {
            log_error("Database error liking post", err);
            return Response::empty(Status::InternalServerError);
        }
    }
}

pub async fn comment(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let query = unwrap_or_return!(request.query.as_ref(), Status::BadRequest);
    let post_id = unwrap_or_return!(validate_like_query(query), Status::BadRequest);

    let comment = match extract_json::<CommentDTO>(request) {
        Ok(res) => res,
        Err(status) => return Response::empty(status),
    };

    match GalleryRepo::post_comment(&state.db, user_id, post_id, &comment.comment).await {
        Ok(_) => {
			let result = GalleryRepo::get_commenter_email(&state.db, user_id, post_id).await;
                
            if let Ok(Some(data)) = result {
                let email_conf = state.email_conf.clone();
                let NotificationData {
                    author_email,
                    commenter_username,
                } = data;

                tokio::spawn(async move {
                    prepare_email(
                        email_conf,
                        author_email,
                        commenter_username,
                        comment.comment,
                    ).await;
                });
            }

            Response::empty(Status::Ok)
        }
        Err(err) => {
            log_error("Database error saving comment post", err);
            Response::empty(Status::InternalServerError)
        }
    }
}

async fn prepare_email(
    email_conf: EmailConfig,
    recv_email: String,
    username: String,
    comment: String,
) {
    let from = format!("Camagru Admin <{}>", email_conf.get_email());
    let to = format!("<{}>", recv_email);
    let subject = "Your post was commented".to_string();
    let body = format!(
        "Hey, {} just left a comment to your post:\n {}",
        username, comment
    );
    send_email(email_conf, from, to, subject, body).await
}

fn validate_gallery_query(query: &str) -> Option<(usize, usize)> {
    let params: Vec<&str> = query.split('&').collect();
    let mut page = None;
    let mut per_page = None;

    for param in params {
        let mut key_value = param.splitn(2, '=');
        let key = key_value.next().unwrap_or("");
        let value = key_value.next().unwrap_or("");

        match key {
            "page" => page = value.parse::<usize>().ok(),
            "per_page" => per_page = value.parse::<usize>().ok(),
            _ => return None,
        }
    }

    if let (Some(page), Some(per_page)) = (page, per_page) {
        Some((page, per_page))
    } else {
        None
    }
}

fn validate_like_query(query: &str) -> Option<i32> {
    let mut key_value = query.splitn(2, '=');
    let key = key_value.next().unwrap_or("");
    let value = key_value.next().unwrap_or("");

    match key {
        "post_id" => Some(value.parse::<i32>().ok()?),
        _ => None,
    }
}