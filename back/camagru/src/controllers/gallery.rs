use crate::dto::gallery_dto::{CommentDTO, GalleryDTO, PaginatedGalleryDTO};
use crate::headers::{log_error, Request, Response, Status};
use crate::server::AppState;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx;
use std::env;
use std::sync::Arc;

pub async fn gallery(request: &Request) -> Response {
    let query = match request.query.as_ref() {
        Some(query) => query,
        None => return Response::empty(Status::BadRequest),
    };

    let (page, per_page) = match validate_gallery_query(query) {
        Some((page, per_page)) => (page, per_page),
        None => return Response::empty(Status::BadRequest),
    };
    println!("Page: {}, Per Page: {}", page, per_page);
    let mut posts: Vec<GalleryDTO> = Vec::new();

    let total_posts: usize = 200;
    let safe_per_page = if per_page > 50 { 50 } else { per_page };
    let current_page = if page == 0 { 1 } else { page };

    let start_index = (current_page - 1) * safe_per_page;
    let end_index = start_index + safe_per_page;
    for i in start_index..end_index.min(total_posts) {
        let post = GalleryDTO {
            author: format!("User {}", i),
            likes: i,
            img_name: String::from("my_new_photo.png"),
            post_id: i + 100,
        };
        posts.push(post);
    }

    let response_data = PaginatedGalleryDTO { posts, total_posts };

    let json = serde_json::to_string(&response_data);
    let respone = match json {
        Ok(json) => Response::json(json),
        Err(e) => {
            log_error("Error in PaginatedGallery serialization", e);
            Response::empty(Status::InternalServerError)
        }
    };

    respone
}

pub async fn like(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };

    let query = match request.query.as_ref() {
        Some(query) => query,
        None => return Response::empty(Status::BadRequest),
    };

    let post_id = match validate_like_query(query) {
        Some(post_id) => post_id,
        None => return Response::empty(Status::BadRequest),
    };

    let mut tx = match state.db.begin().await {
        Ok(transaction) => transaction,
        Err(err) => {
            log_error("Failed to start transaction", err);
            return Response::empty(Status::InternalServerError);
        }
    };

    let q = "DELETE FROM post_likes WHERE user_id = $1 AND post_id = $2";
    let result = sqlx::query(q)
        .bind(&user_id)
        .bind(&post_id)
        .execute(&mut *tx)
        .await;
    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                tx.commit().await;
                return Response::empty(Status::Ok);
            }
            let q = "INSERT INTO post_likes (user_id, post_id) VALUES ($1, $2)";
            let result = sqlx::query(q)
                .bind(&user_id)
                .bind(&post_id)
                .execute(&mut *tx)
                .await;
            match result {
                Ok(_) => {
                    tx.commit().await;
                    return Response::empty(Status::Ok);
                }
                Err(err) => {
                    tx.rollback();
                    log_error("Database error liking post", err);
                    return Response::empty(Status::InternalServerError);
                }
            }
        }
        Err(err) => {
            tx.rollback();
            log_error("Database error deleting like", err);
            return Response::empty(Status::InternalServerError);
        }
    }
}

#[derive(sqlx::FromRow)]
struct NotificationData {
    author_email: String,
    commenter_username: String,
}

pub async fn comment(request: &Request, state: &Arc<AppState>) -> Response {
    let user_id = match request.user_id {
        Some(user_id) => user_id,
        None => return Response::cookie(Status::Unauthorized, "".to_string()),
    };
    let query = match request.query.as_ref() {
        Some(query) => query,
        None => return Response::empty(Status::BadRequest),
    };

    let post_id = match validate_like_query(query) {
        Some(post_id) => post_id,
        None => return Response::empty(Status::BadRequest),
    };
    let body = match request.body.as_ref() {
        Some(body) => body,
        None => return Response::empty(Status::BadRequest),
    };
    let comment = match serde_json::from_slice::<CommentDTO>(body) {
        Ok(res) => res,
        Err(_) => return Response::empty(Status::BadRequest),
    };

    let q = "INSERT INTO comments (user_id, post_id, comment) VALUES ($1, $2, $3)";
    let result = sqlx::query(q)
        .bind(&user_id)
        .bind(&post_id)
        .bind(&comment.comment)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            let q = "SELECT 
                    u_author.email as author_email, 
                    u_commenter.username as commenter_name 
                FROM posts p 
                JOIN users u_author ON u_author.id = p.user_id
                JOIN users u_commenter ON u_commenter.id = $1
                WHERE p.id = $2";
            let result = sqlx::query_as::<_, NotificationData>(q)
                .bind(&user_id)
                .bind(&post_id)
                .fetch_optional(&state.db)
                .await;
            if let Ok(Some(data)) = result {
                let comment_text = comment.comment.clone();
                tokio::spawn(async move {
                    send_email(data.author_email, data.commenter_username, comment_text).await;
                });
            }

            return Response::empty(Status::Ok);
        }
        Err(err) => {
            log_error("Database error saving comment post", err);
            return Response::empty(Status::InternalServerError);
        }
    }
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

async fn send_email(user_email: String, username: String, comment: String) {
    let message = format!(
        "Hey, {} just left a comment to your post:\n {}",
        username, comment
    );
    let (username, password) = parse_env();

    let email = Message::builder()
        .from(format!("Camagru Admin <{}>", username).parse().unwrap())
        .to(format!("<{}>", user_email).parse().unwrap())
        .subject("Someone just commented your post")
        .header(ContentType::TEXT_PLAIN)
        .body(message)
        .unwrap();

    println!("{}, {}", username, password);
    let creds = Credentials::new(username, password);

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

    match mailer.send(email).await {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => eprintln!("Could not send email: {:?}", e),
    }
}

fn parse_env() -> (String, String) {
    let username = match env::var("EMAIL_HOST") {
        Ok(str) => str,
        Err(_) => "default@gmail.com".to_string(),
    };
    let password = match env::var("PASSWORD_HOST") {
        Ok(str) => str,
        Err(_) => "123345".to_string(),
    };
    (username, password)
}

// fn validate_query(params: Vec<String>) -> Option<Vec<usize>> {
// 	let mut key_value = param.splitn(2, '=');
//     let key = key_value.next().unwrap_or("");
//     let value = key_value.next().unwrap_or("");

//     match key {
//         "id" => page = value.parse::<usize>().ok(),
//         _ => return None,
//     }
// }
