use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use regex::Regex;
use serde::Deserialize;
use sqlx::{PgExecutor, PgPool, Postgres};
use std::sync::LazyLock;
use std::time::Duration;
use uuid::Uuid;
use validator::Validate;

use crate::http::{Error, Result};

pub type UserId = Uuid;

pub fn router() -> Router<PgPool> {
    Router::new().route("/v1/user", post(create_user))
}

static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9A-Za-z_]+$").unwrap());

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserAuth {
    #[validate(length(min = 3, max = 16), regex(path= USERNAME_REGEX))]
    username: String,
    password: String,
}

impl UserAuth {
    pub async fn verify(self, db: impl PgExecutor<'_> + Send) -> Result<UserId> {
        self.validate()?;
        let maybe_user = sqlx::query!(
            r#"select user_id,password_hash from "user" where username=$1"#,
            self.username
        )
        .fetch_optional(db)
        .await?;

        if let Some(user) = maybe_user {
            let verified = crate::password::verify(self.password, user.password_hash).await?;

            if verified {
                return Ok(user.user_id);
            }
        }

        let sleep_duration =
            rand::random_range(Duration::from_millis(100)..=Duration::from_millis(500));
        tokio::time::sleep(sleep_duration).await;

        Err(Error::UnprocessableEntity(
            "invalid username/password".into(),
        ))
    }
}

async fn create_user(db: State<PgPool>, Json(req): Json<UserAuth>) -> Result<StatusCode> {
    req.validate()?;

    let UserAuth { username, password } = req;

    let password_hash = crate::password::hash(password).await?;

    sqlx::query!(
        r#"insert into "user"(username, password_hash) values ($1, $2)"#,
        username,
        password_hash
    )
    .execute(&*db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(dbe) if dbe.constraint() == Some("user_username_key") => {
            Error::Conflict("username taken".into())
        }
        _ => e.into(),
    })?;

    Ok(StatusCode::NO_CONTENT)
}
