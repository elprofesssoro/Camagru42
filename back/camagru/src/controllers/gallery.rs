use crate::dto::gallery_dto::{CommentDTO, PaginatedGalleryDTO, PaginationQuery, PostIdQuery};
use crate::headers::{Request, Response, Status};
use crate::unwrap_or_return;
use crate::repositories::gallery_repo::{GalleryRepo, NotificationData};
use crate::utils::{extract_json, log_error, send_email, AppState, EmailConfig};
use std::sync::Arc;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::extract::{Json, State, Extension, Query};
use axum_extra::extract::cookie::{Cookie, CookieJar};

pub async fn gallery(State(state): State<Arc<AppState>>, Query(query): Query<PaginationQuery>) -> AxumResponse {
	let per_page = query.per_page.unwrap_or(5);
    let page = query.page.unwrap_or(1);
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
			(StatusCode::OK, Json(response_data)).into_response()
        }
        Err(e) => {
            log_error("Database error fetching paginated posts", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn like(State(state): State<Arc<AppState>>, Extension(user_id): Extension<i32>, Query(query): Query<PostIdQuery>) -> impl IntoResponse {
   
    match GalleryRepo::toggle_like(&state.db, user_id, query.post_id).await {
        Ok(false) => StatusCode::OK,
        Ok(true) => StatusCode::CREATED,
        Err(err) => {
            log_error("Database error liking post", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn comment(State(state): State<Arc<AppState>>, Extension(user_id): Extension<i32>, Query(query): Query<PostIdQuery>, Json(payload): Json<CommentDTO>) -> impl IntoResponse {
    match GalleryRepo::post_comment(&state.db, user_id, query.post_id, &payload.comment).await {
        Ok(_) => {
			let result = GalleryRepo::get_commenter_email(&state.db, user_id, query.post_id).await;
                
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
                        payload.comment,
                    ).await;
                });
            }

            StatusCode::OK
        }
        Err(err) => {
            log_error("Database error saving comment post", err);
            StatusCode::INTERNAL_SERVER_ERROR
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_gallery_query() {
        assert_eq!(validate_gallery_query("page=1&per_page=10"), Some((1, 10)));
        assert_eq!(validate_gallery_query("per_page=25&page=3"), Some((3, 25)));

        assert_eq!(validate_gallery_query("page=1"), None);
        assert_eq!(validate_gallery_query("per_page=10"), None);
        assert_eq!(validate_gallery_query(""), None);

        assert_eq!(validate_gallery_query("page=eins&per_page=10"), None);
        assert_eq!(validate_gallery_query("page=1&per_page=zehn"), None);

        assert_eq!(validate_gallery_query("page=1&per_page=10&sort=desc"), None);
        assert_eq!(validate_gallery_query("random=123"), None);
    }

    #[test]
    fn test_validate_like_query() {
        assert_eq!(validate_like_query("post_id=42"), Some(42));
        assert_eq!(validate_like_query("post_id=0"), Some(0));

        assert_eq!(validate_like_query("post_id=abc"), None);
        assert_eq!(validate_like_query("post_id="), None);

        assert_eq!(validate_like_query("id=42"), None);
        assert_eq!(validate_like_query("post=42"), None);
        assert_eq!(validate_like_query(""), None);
    }
}