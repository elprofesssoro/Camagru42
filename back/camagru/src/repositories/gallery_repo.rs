use crate::dto::gallery_dto::GalleryDTO;
use sqlx::{PgPool, Error, FromRow, QueryBuilder, postgres::PgQueryResult, Postgres};

#[derive(FromRow)]
pub struct NotificationData {
    pub author_email: String,
    pub commenter_username: String,
}

pub struct GalleryRepo;

impl GalleryRepo {
	pub async fn get_paginated_posts(db: &PgPool, limit: i64, offset: i64) -> Result<(i64, Vec<GalleryDTO>), Error> {
        let q_count = "SELECT COUNT(*) FROM posts";
        let q_posts = "
            SELECT 
                COALESCE(u.username, '[Deleted User]') AS author,
                p.id AS post_id, 
                p.image_path AS img_name,
                (SELECT COUNT(*) FROM post_likes WHERE post_id = p.id) AS likes
            FROM posts p
            LEFT JOIN users u ON p.user_id = u.id
            ORDER BY p.post_date DESC
            LIMIT $1 OFFSET $2
        ";

        tokio::try_join!(
            sqlx::query_scalar::<_, i64>(q_count).fetch_one(db),
            sqlx::query_as::<_, GalleryDTO>(q_posts)
                .bind(limit)
                .bind(offset)
                .fetch_all(db)
        )
    }

	pub async fn toggle_like(db: &PgPool, user_id: i32, post_id: i32) -> Result<bool, Error> {
		let mut tx = db.begin().await?;
		let q = "DELETE FROM post_likes WHERE user_id = $1 AND post_id = $2";
    	let result = sqlx::query(q)
    	    .bind(&user_id)
    	    .bind(&post_id)
    	    .execute(&mut *tx)
    	    .await?;

		if result.rows_affected() > 0 {
              tx.commit().await;
              return Ok(false);
          }
        let q = "INSERT INTO post_likes (user_id, post_id) VALUES ($1, $2)";
        sqlx::query(q)
            .bind(&user_id)
            .bind(&post_id)
            .execute(&mut *tx)
            .await?;

		tx.commit().await?;
		Ok(true)
	}

	pub async fn post_comment(db: &PgPool, user_id: i32, post_id: i32, comment: &str) -> Result<(), Error> {
		let q = "INSERT INTO comments (user_id, post_id, comment) VALUES ($1, $2, $3)";
    	sqlx::query(q)
    	    .bind(user_id)
    	    .bind(post_id)
    	    .bind(comment)
    	    .execute(db)
    	    .await?;

		Ok(())
	}

	pub async fn get_commenter_email(db: &PgPool, user_id: i32, post_id: i32) -> Result<Option<NotificationData>, Error> {
		let q = "SELECT 
                u_author.email as author_email, 
                u_commenter.username as commenter_username 
            FROM posts p 
            JOIN users u_author ON u_author.id = p.user_id
            JOIN users u_commenter ON u_commenter.id = $1
            WHERE p.id = $2 
              AND u_author.notify_comment = TRUE 
              AND u_author.is_deleted = FALSE";
              
        sqlx::query_as::<_, NotificationData>(q)
            .bind(user_id)
            .bind(post_id)
            .fetch_optional(db)
            .await
	} 

}