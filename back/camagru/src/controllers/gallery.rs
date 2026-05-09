use crate::dto::gallery_dto::{CommentDTO, PaginatedGalleryDTO, PaginationQuery, PostIdQuery};
use crate::repositories::gallery_repo::{GalleryRepo, NotificationData};
use crate::utils::{log_error, send_email, AppState, EmailConfig};
use std::sync::Arc;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::extract::{Json, State, Extension, Query};

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