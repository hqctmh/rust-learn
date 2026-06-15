#[cfg(feature = "ssr")]
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use sha2::{Digest, Sha256};
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
    const LEGACY_SHA256_PASSWORD_HASH_PREFIX: &'static str = "sha256$v1$";

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
        let matches = if stored_password.starts_with("$argon2") {
            Self::verify_argon2_password(stored_password, supplied_password)
        } else if stored_password.starts_with(Self::LEGACY_SHA256_PASSWORD_HASH_PREFIX) {
            stored_password == Self::legacy_sha256_password_hash(supplied_password)
        } else {
            stored_password == supplied_password
        };

        if !matches {
            return Err(ForumError::Unauthorized);
        }

        Ok(())
    }

    pub fn hash_password(password: &str) -> String {
        Self::argon2_password_hash(password)
    }

    #[cfg(feature = "ssr")]
    fn argon2_password_hash(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("argon2 password hashing should work with generated salt")
            .to_string()
    }

    #[cfg(not(feature = "ssr"))]
    fn argon2_password_hash(password: &str) -> String {
        Self::legacy_sha256_password_hash(password)
    }

    #[cfg(feature = "ssr")]
    fn verify_argon2_password(stored_password: &str, supplied_password: &str) -> bool {
        PasswordHash::new(stored_password)
            .ok()
            .and_then(|parsed| {
                Argon2::default()
                    .verify_password(supplied_password.as_bytes(), &parsed)
                    .ok()
            })
            .is_some()
    }

    #[cfg(not(feature = "ssr"))]
    fn verify_argon2_password(_stored_password: &str, _supplied_password: &str) -> bool {
        false
    }

    fn legacy_sha256_password_hash(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"post-forum-password:v1:");
        hasher.update(password.as_bytes());
        format!(
            "{}{:x}",
            Self::LEGACY_SHA256_PASSWORD_HASH_PREFIX,
            hasher.finalize()
        )
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
