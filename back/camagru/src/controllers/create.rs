use crate::dto::create_dto::{CreatePostDTO, HistoryDTO, PostDetailsDTO, CommentDTO, PostIdQuery};
use crate::headers::{Request, Response, Status};
use crate::unwrap_or_return;
use crate::repositories::create_repo::CreateRepo;
use crate::utils::{extract_json, log_error, AppState};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::sync::Arc;
use sqlx::Row;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::extract::{Json, State, Extension, Query};
use axum_extra::extract::cookie::{Cookie, CookieJar};

pub async fn create_post(State(state): State<Arc<AppState>>, Extension(user_id): Extension<i32>, Json(payload): Json<CreatePostDTO>) -> impl IntoResponse {

    let image_str = payload
        .image
        .split_once(',')
        .map(|(_, data)| data.to_string())
        .unwrap_or(payload.image);
        
    let sticker_name = payload.sticker_name;
    let (pos_x, pos_y, width, height) = (payload.pos_x, payload.pos_y, payload.width, payload.height);
    let pub_path = state.img_root_dir.clone();

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
        Ok(Err(status)) => return status,
        Err(err) => {
            log_error("Error joining thread for image processing", err);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    match CreateRepo::create_post(&state.db, user_id, &image_name).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            log_error("Database error saving post", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn create_get(State(state): State<Arc<AppState>>, Extension(user_id): Extension<i32>) -> AxumResponse {
        let posts: Vec<HistoryDTO> = match CreateRepo::get_user_posts(&state.db, user_id).await {
        Ok(posts) => posts,
        Err(e) => {
            log_error("Database error fetching user posts", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

	(StatusCode::OK, Json(posts)).into_response()
}

pub async fn create_delete(State(state): State<Arc<AppState>>, Extension(user_id): Extension<i32>, Query(query): Query<PostIdQuery>) -> impl IntoResponse {
    
    let post_id = query.post_id;

    match CreateRepo::delete_post(&state.db, post_id, user_id).await {
        Ok(Some(image_path)) => {
            let full_file_path = format!("{}/posts/{}", state.img_root_dir, image_path);
            
            if let Err(e) = tokio::fs::remove_file(&full_file_path).await {
                log_error("Warning: Failed to delete image file from disk", e);
            }
            StatusCode::OK
        }
        Ok(None) => StatusCode::NOT_FOUND,
        Err(err) => {
            log_error("Error deleting post", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn create_details(State(state): State<Arc<AppState>>, Query(query): Query<PostIdQuery>) -> AxumResponse {
    let post_id = query.post_id;

    match CreateRepo::get_post_details(&state.db, post_id).await {
        Ok((post_details, comments)) => {
            let response = PostDetailsDTO {
                post_date: post_details.get("post_date"),
                likes: post_details.get("likes"),
                comments,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            log_error("Database error fetching post details", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
) -> Result<String, StatusCode> {
    let image_bytes = match STANDARD.decode(image_str) {
        Ok(b) => b,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let mut img = match image::load_from_memory(&image_bytes) {
        Ok(i) => i,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let sticker_path = format!("{}/stickers/{}", pub_path, sticker_name);
    let sticker = match image::open(&sticker_path) {
        Ok(s) => s,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    let sticker = image::imageops::resize(&sticker, width, height, image::imageops::FilterType::Nearest);
    image::imageops::overlay(&mut img, &sticker, pos_x, pos_y);

    let img_name = format!("{}.jpg", uuid::Uuid::new_v4());
    let final_path = format!("{}/posts/{}", pub_path, img_name);
    
    if img.save(&final_path).is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    const TINY_BASE64_IMAGE: &str = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////wgALCAABAAEBAREA/8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQABPxA=";

    #[test]
    fn test_process_image_invalid_base64() {
        let result = process_image(
            "nothing", 
            "./dummy_path", 
            "sticker.png", 
            100, 100, 0, 0
        );
        assert_eq!(result, Err(Status::BadRequest));
    }

    #[test]
    fn test_process_image_missing_sticker() {
        let result = process_image(
            TINY_BASE64_IMAGE, 
            "./dummy_path", 
            "nothing.png", 
            100, 100, 0, 0
        );
        assert_eq!(result, Err(Status::NotFound));
    }

    #[test]
    fn test_process_image_success() {
        let test_dir = "./test_pub_path";
        let stickers_dir = format!("{}/stickers", test_dir);
        let posts_dir = format!("{}/posts", test_dir);
        
        fs::create_dir_all(&stickers_dir).unwrap();
        fs::create_dir_all(&posts_dir).unwrap();

        let sticker_path = format!("{}/test_sticker.jpg", stickers_dir);
        let image_bytes = STANDARD.decode(TINY_BASE64_IMAGE).unwrap();
        fs::write(&sticker_path, &image_bytes).unwrap();

        let result = process_image(
            TINY_BASE64_IMAGE,
            test_dir,
            "test_sticker.jpg",
            1, 1, 0, 0
        );

        assert!(result.is_ok(), "Picture should be processed");
        
        let saved_image_name = result.unwrap();
        let expected_path = format!("{}/{}", posts_dir, saved_image_name);
        
        assert!(Path::new(&expected_path).exists(), "Data was not found");

        fs::remove_dir_all(test_dir).unwrap();
    }
}