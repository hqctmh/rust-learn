use crate::{
    domain::users::{ChangePasswordRequest, UpdateAvatarRequest, UpdateProfileRequest},
    error::ForumError,
    services::auth::AuthService,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedProfile {
    pub nickname: String,
    pub bio: String,
}

pub struct UserSettingsService;

impl UserSettingsService {
    pub fn normalize_profile(
        request: UpdateProfileRequest,
    ) -> Result<NormalizedProfile, ForumError> {
        let nickname = request.nickname.trim();
        let bio = request.bio.trim();
        if nickname.is_empty() {
            return Err(ForumError::Validation("昵称不能为空".to_string()));
        }
        if nickname.chars().count() > 32 {
            return Err(ForumError::Validation("昵称不能超过 32 个字符".to_string()));
        }
        if bio.chars().count() > 160 {
            return Err(ForumError::Validation(
                "简介不能超过 160 个字符".to_string(),
            ));
        }

        Ok(NormalizedProfile {
            nickname: nickname.to_string(),
            bio: bio.to_string(),
        })
    }

    pub fn normalize_avatar(request: UpdateAvatarRequest) -> Result<String, ForumError> {
        let avatar_url = request.avatar_url.trim();
        if avatar_url.is_empty()
            || !(avatar_url.starts_with('/')
                || avatar_url.starts_with("http://")
                || avatar_url.starts_with("https://"))
        {
            return Err(ForumError::Validation("头像 URL 不合法".to_string()));
        }

        Ok(avatar_url.to_string())
    }

    pub fn validate_password_change(
        stored_password: &str,
        request: ChangePasswordRequest,
    ) -> Result<String, ForumError> {
        let old_password = request.old_password.trim();
        let new_password = request.new_password.trim();
        if old_password.is_empty() || new_password.is_empty() {
            return Err(ForumError::Validation("原密码和新密码不能为空".to_string()));
        }
        if new_password.chars().count() < 6 {
            return Err(ForumError::Validation(
                "新密码不能少于 6 个字符".to_string(),
            ));
        }
        AuthService::validate_password_match(stored_password, old_password)?;

        Ok(new_password.to_string())
    }
}
