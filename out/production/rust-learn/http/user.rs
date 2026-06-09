use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use regex::Regex;
use serde::Deserialize;
use sqlx::{PgExecutor, PgPool, Postgres};
use std::sync::LazyLock;
use uuid::Uuid;
use validator::Validate;

use crate::http::{Error, Result};

pub type UserId = Uuid;

pub fn router() -> Router<PgPool> {
    Router::new().route("/v1/user", post(create_user))
}

static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9A-Za-z_]+$").unwrap());
const SELECT_USER_ID_BY_USERNAME: &str = concat!(
    r#"select user_id from "#,
    r#""user""#,
    r#" where username = $1"#
);

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
        let maybe_user = sqlx::query_scalar::<Postgres, UserId>(SELECT_USER_ID_BY_USERNAME)
            .bind(self.username)
            .fetch_optional(db)
            .await?;

        maybe_user
            .ok_or_else(|| Error::UnprocessableEntity("invalid username or password".to_owned()))
    }
}

async fn create_user(State(_db): State<PgPool>, Json(_req): Json<UserAuth>) -> Result<StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}
