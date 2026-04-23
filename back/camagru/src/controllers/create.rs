use crate::dto::create_dto::{CreatePostDTO, HistoryDTO, PostDetailsDTO, CommentDTO};
use crate::headers::{Request, Response, Status};
use crate::unwrap_or_return;
use crate::utils::{extract_json, log_error, AppState};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::sync::Arc;
use sqlx::Row;

pub async fn create_post(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let payload = match extract_json::<CreatePostDTO>(request) {
        Ok(res) => res,
        Err(status) => return Response::empty(status),
    };

    let image_str = payload
        .image
        .split_once(',')
        .map(|(_, data)| data.to_string())
        .unwrap_or(payload.image);
        
    let sticker_name = payload.sticker_name;
    let (pos_x, pos_y, width, height) = (payload.pos_x, payload.pos_y, payload.width, payload.height);
    let pub_path = request.pub_path.clone();

    let process_result = tokio::task::spawn_blocking(move || {
        process_image(
            &image_str,
            &pub_path,
            &sticker_name,
            width,
            height,
            pos_x as i64,
            pos_y as i64,
        )
    })
    .await;

    let image_name = match process_result {
        Ok(Ok(image_name)) => {
            println!("Image successfully processed in background and saved to {}", image_name);
            image_name
        }
        Ok(Err(status)) => return Response::empty(status),
        Err(err) => {
            log_error("Error joining thread for image processing", err);
            return Response::empty(Status::InternalServerError);
        }
    };

    let q = "INSERT INTO posts (user_id, image_path) VALUES ($1, $2)";
    let result = sqlx::query(q)
        .bind(user_id)
        .bind(&image_name)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Response::empty(Status::Ok),
        Err(err) => {
            log_error("Database error saving post", err);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn create_get(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let q = "SELECT image_path, post_date, id FROM posts WHERE user_id = $1";
    let posts: Vec<HistoryDTO> = match sqlx::query_as::<_, HistoryDTO>(q)
        .bind(user_id)
        .fetch_all(&state.db)
        .await
    {
        Ok(posts) => posts,
        Err(e) => {
            log_error("Database error fetching user posts", e);
            return Response::empty(Status::InternalServerError);
        }
    };

    match serde_json::to_string(&posts) {
        Ok(json) => Response::json(json),
        Err(e) => {
            log_error("Error in HistoryDTO serialization", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn create_delete(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let query = unwrap_or_return!(&request.query, Status::BadRequest);
    let post_id = unwrap_or_return!(validate_delete_query(query), Status::BadRequest);

    let q = "DELETE FROM posts WHERE id = $1 AND user_id = $2 RETURNING image_path";
    let result = sqlx::query(q)
        .bind(post_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(row)) => {
            let image_path: String = row.get("image_path");
            let full_file_path = format!("{}/posts/{}", request.pub_path, image_path);
            
            if let Err(e) = tokio::fs::remove_file(&full_file_path).await {
                log_error("Warning: Failed to delete image file from disk", e);
            }
            Response::empty(Status::Ok)
        }
        Ok(None) => Response::empty(Status::NotFound),
        Err(err) => {
            log_error("Error deleting post", err);
            Response::empty(Status::InternalServerError)
        }
    }
}

pub async fn create_details(request: &Request, state: &Arc<AppState>) -> Response {
    let _user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let query = unwrap_or_return!(&request.query, Status::BadRequest);
    let post_id = unwrap_or_return!(validate_delete_query(query), Status::BadRequest);

    let q_post = "
        SELECT 
            post_date,
            (SELECT COUNT(*) FROM post_likes WHERE post_id = $1) AS likes 
        FROM posts WHERE id = $1;";

    let q_comments = "
        SELECT 
            COALESCE(u.username, '[Deleted User]') AS username,
            c.comment 
        FROM comments c
        LEFT JOIN users u ON c.user_id = u.id
        WHERE c.post_id = $1
        ORDER BY c.id DESC;";

    let result = tokio::try_join!(
        sqlx::query(q_post).bind(&post_id).fetch_one(&state.db),
        sqlx::query_as::<_, CommentDTO>(q_comments).bind(&post_id).fetch_all(&state.db)
    );

    match result {
        Ok((post_details, comments)) => {
            let response = PostDetailsDTO {
                post_date: post_details.get("post_date"),
                likes: post_details.get("likes"),
                comments,
            };

            match serde_json::to_string(&response) {
                Ok(json) => Response::json(json),
                Err(e) => {
                    log_error("Error in PostDetails serialization", e);
                    Response::empty(Status::InternalServerError)
                }
            }
        }
        Err(sqlx::Error::RowNotFound) => Response::empty(Status::NotFound),
        Err(e) => {
            log_error("Database error fetching post details", e);
            Response::empty(Status::InternalServerError)
        }
    }
}

fn process_image(
    image_str: &str,
    pub_path: &str,
    sticker_name: &str,
    width: u32,
    height: u32,
    pos_x: i64,
    pos_y: i64,
) -> Result<String, Status> {
    let image_bytes = match STANDARD.decode(image_str) {
        Ok(b) => b,
        Err(_) => return Err(Status::BadRequest),
    };

    let mut img = match image::load_from_memory(&image_bytes) {
        Ok(i) => i,
        Err(_) => return Err(Status::BadRequest),
    };

    let sticker_path = format!("{}/stickers/{}", pub_path, sticker_name);
    let sticker = match image::open(&sticker_path) {
        Ok(s) => s,
        Err(_) => return Err(Status::NotFound),
    };

    let sticker = image::imageops::resize(&sticker, width, height, image::imageops::FilterType::Nearest);
    image::imageops::overlay(&mut img, &sticker, pos_x, pos_y);

    let img_name = format!("{}.jpg", uuid::Uuid::new_v4());
    let final_path = format!("{}/posts/{}", pub_path, img_name);
    
    if img.save(&final_path).is_err() {
        return Err(Status::InternalServerError);
    }

    Ok(img_name)
}

pub fn validate_delete_query(query: &str) -> Option<i32> {
    let mut key_value = query.splitn(2, '=');
    let key = key_value.next().unwrap_or("");
    let value = key_value.next().unwrap_or("");

    match key {
        "post_id" => value.parse::<i32>().ok(),
        _ => None,
    }
}