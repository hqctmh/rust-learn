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

        

        Ok(())
    }
}
