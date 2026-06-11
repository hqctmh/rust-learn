use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::auth::{Session, SessionUser};

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct UserAuthRow {
    pub user_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub is_admin: bool,
}

impl UserAuthRow {
    pub fn session_user(&self) -> SessionUser {
        SessionUser {
            user_id: self.user_id,
            username: self.username.clone(),
            nickname: self.nickname.clone(),
            avatar_url: self.avatar_url.clone(),
            is_admin: self.is_admin,
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.status == "disabled"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct SessionAuthRow {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub is_admin: bool,
    pub expires_at: OffsetDateTime,
}

impl SessionAuthRow {
    pub fn session_user(&self) -> SessionUser {
        SessionUser {
            user_id: self.user_id,
            username: self.username.clone(),
            nickname: self.nickname.clone(),
            avatar_url: self.avatar_url.clone(),
            is_admin: self.is_admin,
        }
    }

    pub fn session(&self) -> Session {
        Session {
            session_id: self.session_id,
            user: self.session_user(),
            expires_at: self.expires_at,
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.status == "disabled"
    }
}

pub struct PostgresAuthRepository;

impl PostgresAuthRepository {
    pub fn find_user_by_username_sql() -> &'static str {
        r#"
select
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    status,
    is_admin
from users
where username = $1
limit 1
"#
    }

    pub fn find_user_by_id_sql() -> &'static str {
        r#"
select
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    status,
    is_admin
from users
where user_id = $1
limit 1
"#
    }

    pub fn insert_user_sql() -> &'static str {
        r#"
insert into users (
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    is_admin
)
values ($1, $2, $3, $4, $5, $6)
returning
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    status,
    is_admin
"#
    }

    pub fn insert_session_sql() -> &'static str {
        r#"
insert into sessions (
    session_id,
    user_id,
    token_hash,
    expires_at
)
values ($1, $2, $3, $4)
"#
    }

    pub fn find_session_sql() -> &'static str {
        r#"
select
    s.session_id,
    u.user_id,
    u.username,
    u.nickname,
    u.avatar_url,
    u.status,
    u.is_admin,
    s.expires_at
from sessions s
join users u on u.user_id = s.user_id
where s.session_id = $1
limit 1
"#
    }

    pub fn delete_session_sql() -> &'static str {
        r#"
delete from sessions
where session_id = $1
"#
    }

    pub async fn find_user_by_username(
        pool: &sqlx::PgPool,
        username: &str,
    ) -> sqlx::Result<Option<UserAuthRow>> {
        sqlx::query_as!(
            UserAuthRow,
            r#"
select
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    status,
    is_admin
from users
where username = $1
limit 1
"#,
            username
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_user_by_id(
        pool: &sqlx::PgPool,
        user_id: Uuid,
    ) -> sqlx::Result<Option<UserAuthRow>> {
        sqlx::query_as!(
            UserAuthRow,
            r#"
select
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    status,
    is_admin
from users
where user_id = $1
limit 1
"#,
            user_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn insert_user(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        username: &str,
        password_hash: &str,
        nickname: &str,
        avatar_url: Option<&str>,
        is_admin: bool,
    ) -> sqlx::Result<UserAuthRow> {
        sqlx::query_as!(
            UserAuthRow,
            r#"
insert into users (
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    is_admin
)
values ($1, $2, $3, $4, $5, $6)
returning
    user_id,
    username,
    password_hash,
    nickname,
    avatar_url,
    status,
    is_admin
"#,
            user_id,
            username,
            password_hash,
            nickname,
            avatar_url,
            is_admin
        )
        .fetch_one(pool)
        .await
    }

    pub async fn insert_session(
        pool: &sqlx::PgPool,
        session_id: Uuid,
        user_id: Uuid,
        token_hash: &str,
        expires_at: OffsetDateTime,
    ) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            r#"
insert into sessions (
    session_id,
    user_id,
    token_hash,
    expires_at
)
values ($1, $2, $3, $4)
"#,
            session_id,
            user_id,
            token_hash,
            expires_at
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn find_session(
        pool: &sqlx::PgPool,
        session_id: Uuid,
    ) -> sqlx::Result<Option<SessionAuthRow>> {
        sqlx::query_as!(
            SessionAuthRow,
            r#"
select
    s.session_id,
    u.user_id,
    u.username,
    u.nickname,
    u.avatar_url,
    u.status,
    u.is_admin,
    s.expires_at
from sessions s
join users u on u.user_id = s.user_id
where s.session_id = $1
limit 1
"#,
            session_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn delete_session(pool: &sqlx::PgPool, session_id: Uuid) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            r#"
delete from sessions
where session_id = $1
"#,
            session_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
