use anyhow::anyhow;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::{domain::model::User, repository::user::UserRepository};

struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub async fn user_regiest(
        &self,
        username: &str,
        password: &str,
        display_name: Option<&str>,
    ) -> anyhow::Result<User> {
        let user = self.user_repository.find_by_username(username).await?;

        if user.is_some() {
            anyhow::bail!("用户名已经存在");
        }

        let password_hash = Self::hash_password(password).await?;

        self.user_repository
            .create_user(username, password_hash.as_str(), display_name)
            .await
    }

    pub async fn user_login(&self, username: &str, password: &str) -> anyhow::Result<User> {
        let user = self
            .user_repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| anyhow!("用户名或密码错误"))?;

        let password = password.to_owned();
        let password_hash = user.password_hash.clone();
        let verify_result = tokio::task::spawn_blocking(move || {
            let stored_hash =
                PasswordHash::new(&password_hash).map_err(|_| anyhow!("用户名或密码错误"))?;

            Argon2::default()
                .verify_password(password.as_bytes(), &stored_hash)
                .map_err(|_| anyhow!("用户名或密码错误"))
        })
        .await
        .map_err(|e| anyhow!("密码验证任务异常:{e}"))?;

        verify_result?;

        Ok(user)
    }

    async fn hash_password(password: &str) -> anyhow::Result<String> {
        let password = password.to_owned();
        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
                .map_err(|e| anyhow!("密码哈希失败:{e}"))
        })
        .await
        .map_err(|e| anyhow!("密码哈希任务异常:{e}"))?
    }
}
