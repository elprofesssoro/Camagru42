use sqlx::{postgres::PgQueryResult, Error, FromRow, PgPool, Postgres, QueryBuilder};

use crate::dto::request_dto::{RegisterDTO, UserInfoDTO};

#[derive(FromRow)]
pub struct UserAuthData {
    pub id: i32,
    pub password: String,
    pub is_verified: bool,
    pub is_deleted: bool,
}

pub struct UserRepo;

impl UserRepo {
    pub async fn get_user(
        db: &PgPool,
        cred: &str,
        search_by: &str,
    ) -> Result<Option<UserAuthData>, Error> {
        let q = format!(
            "SELECT id, password, is_verified, is_deleted FROM users WHERE {} = $1",
            search_by
        );

        sqlx::query_as::<_, UserAuthData>(&q)
            .bind(cred)
            .fetch_optional(db)
            .await
    }

    pub async fn get_password(db: &PgPool, user_id: i32) -> Result<String, Error> {
        let q = "SELECT password FROM users WHERE id = $1";

        let password = sqlx::query_scalar::<_, String>(q)
            .bind(user_id)
            .fetch_one(db)
            .await?;

        Ok(password)
    }

    pub async fn delete_session(db: &PgPool, user_id: i32) -> Result<(), Error> {
        let q = "DELETE FROM sessions WHERE user_id = $1";

        sqlx::query(q).bind(user_id).execute(db).await?;

        Ok(())
    }

    pub async fn register_user(
        db: &PgPool,
        payload: &RegisterDTO,
        v_token: &str,
        hashed: &str,
    ) -> Result<(), Error> {
        let q =
    	    "INSERT INTO users (email, username, password, verification_token) VALUES ($1, $2, $3, $4)";

        sqlx::query(&q)
            .bind(&payload.email)
            .bind(&payload.username)
            .bind(hashed)
            .bind(v_token)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn session_token_insert(
        db: &PgPool,
        session: &str,
        user_id: i32,
    ) -> Result<(), Error> {
        let q = "INSERT INTO sessions (session_token, user_id, expires_at) 
        VALUES ($1, $2, NOW() + INTERVAL '5 minutes')
        ON CONFLICT (user_id) 
        DO UPDATE SET 
            session_token = EXCLUDED.session_token, 
            expires_at = EXCLUDED.expires_at";

        sqlx::query(q)
            .bind(session)
            .bind(user_id)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn verify_user(db: &PgPool, token: &str) -> Result<bool, Error> {
        let q = "UPDATE users SET is_verified = TRUE, verification_token = NULL WHERE verification_token = $1";
        let res = sqlx::query(q).bind(token).execute(db).await?;

        Ok(res.rows_affected() == 1)
    }

    pub async fn reset_pass_req(
        db: &PgPool,
        p_token: &str,
        email: &str,
    ) -> Result<PgQueryResult, Error> {
        let q =
        	"UPDATE users SET reset_verification_token = $1, reset_expires_at = NOW() + INTERVAL '5 minutes' WHERE email = $2";
        sqlx::query(&q).bind(p_token).bind(email).execute(db).await
    }

    pub async fn reset_pass_verify(db: &PgPool, token: &str) -> Result<Option<i32>, Error> {
        let q =
            "SELECT id FROM users WHERE reset_verification_token = $1 AND reset_expires_at > NOW()";
        let id = sqlx::query_scalar::<_, i32>(q)
            .bind(token)
            .fetch_optional(db)
            .await?;
        Ok(id)
    }

    pub async fn reset_pass_update(db: &PgPool, hashed: &str, id: i32) -> Result<(), Error> {
        let q =
      		  	"UPDATE users SET password = $1, reset_verification_token = NULL, reset_expires_at = NULL WHERE id = $2";
        sqlx::query(&q).bind(hashed).bind(id).execute(db).await?;
        Ok(())
    }

    pub async fn resend_email(
        db: &PgPool,
        token: &str,
        email: &str,
    ) -> Result<PgQueryResult, Error> {
        let q = "UPDATE users 
			SET verification_token = $1 
			WHERE email = $2 AND is_verified = FALSE";
        sqlx::query(&q).bind(token).bind(email).execute(db).await
    }

    pub async fn user_info(db: &PgPool, id: i32) -> Result<Option<UserInfoDTO>, Error> {
        let q = "SELECT email, username, notify_comment FROM users WHERE id = $1";
        sqlx::query_as::<_, UserInfoDTO>(q)
            .bind(id)
            .fetch_optional(db)
            .await
    }

    pub async fn update_user(
        db: &PgPool,
        user_id: i32,
        email: Option<&str>,
        username: Option<&str>,
        notify_comment: Option<bool>,
        new_hashed_password: Option<&str>,
    ) -> Result<(), Error> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE users SET ");
        let mut separated = query_builder.separated(", ");

        if let Some(e) = email {
            separated.push("email = ").push_bind_unseparated(e);
        }
        if let Some(u) = username {
            separated.push("username = ").push_bind_unseparated(u);
        }
        if let Some(n) = notify_comment {
            separated.push("notify_comment = ").push_bind_unseparated(n);
        }
        if let Some(p) = new_hashed_password {
            separated.push("password = ").push_bind_unseparated(p);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(user_id);

        query_builder.build().execute(db).await?;

        Ok(())
    }

    pub async fn delete_user(db: &PgPool, user_id: i32) -> Result<(), Error> {
        let dummy_email = format!("deleted_{}@camagru.local", user_id);
        let dummy_username = format!("deleted_{}", user_id);

        let q = "UPDATE users 
    	SET is_deleted = TRUE, 
    	    email = $1,
    	    username = $2,
    	    password = ''
    	WHERE id = $3";

        sqlx::query(q)
            .bind(dummy_email)
            .bind(dummy_username)
            .bind(user_id)
            .execute(db)
            .await?;

        Ok(())
    }
}
