use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::auth::{RegisterRequest, Session, SessionUser},
    error::ForumError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedLogin {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedRegistration {
    pub username: String,
    pub password: String,
    pub nickname: String,
}

pub struct AuthService;

impl AuthService {
    pub fn normalize_login(username: &str, password: &str) -> Result<NormalizedLogin, ForumError> {
        let username = username.trim();
        let password = password.trim();
        if username.is_empty() || password.is_empty() {
            return Err(ForumError::Validation("用户名和密码不能为空".to_string()));
        }

        Ok(NormalizedLogin {
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    pub fn normalize_registration(
        request: RegisterRequest,
    ) -> Result<NormalizedRegistration, ForumError> {
        let username = request.username.trim();
        let password = request.password.trim();
        let nickname = request.nickname.trim();
        if username.is_empty() || password.is_empty() || nickname.is_empty() {
            return Err(ForumError::Validation(
                "用户名、昵称和密码不能为空".to_string(),
            ));
        }
        if username.chars().count() > 32 || nickname.chars().count() > 32 {
            return Err(ForumError::Validation(
                "用户名和昵称不能超过 32 个字符".to_string(),
            ));
        }

        Ok(NormalizedRegistration {
            username: username.to_string(),
            password: password.to_string(),
            nickname: nickname.to_string(),
        })
    }

    pub fn build_login_user(user_id: Uuid, username: &str) -> SessionUser {
        SessionUser {
            user_id,
            username: username.to_string(),
            nickname: username.to_string(),
            avatar_url: None,
            is_admin: username == "admin",
        }
    }

    pub fn build_registered_user(
        user_id: Uuid,
        registration: NormalizedRegistration,
    ) -> SessionUser {
        SessionUser {
            user_id,
            username: registration.username,
            nickname: registration.nickname,
            avatar_url: None,
            is_admin: false,
        }
    }

    pub fn validate_password_match(
        stored_password: &str,
        supplied_password: &str,
    ) -> Result<(), ForumError> {
        if stored_password != supplied_password {
            return Err(ForumError::Unauthorized);
        }

        Ok(())
    }

    pub fn build_session(session_id: Uuid, user: SessionUser, now: OffsetDateTime) -> Session {
        Session {
            session_id,
            user,
            expires_at: now + Duration::days(7),
        }
    }

    pub fn validate_session_active(
        expires_at: OffsetDateTime,
        now: OffsetDateTime,
    ) -> Result<(), ForumError> {
        if expires_at <= now {
            return Err(ForumError::Unauthorized);
        }

        Ok(())
    }
}
