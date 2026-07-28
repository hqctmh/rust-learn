use std::{env, sync::Arc};

use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{domain::service::user::UserService, repository::user::UserRepository};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub user_repository: Arc<UserRepository>,
    pub user_service: Arc<UserService>,
}

impl AppState {
    pub async fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let pool = Self::create_database_pool().await?;

        let user_repository = Arc::new(UserRepository::new(pool.clone()));

        let user_service = Arc::new(UserService::new(Arc::clone(&user_repository)));

        Ok(Self {
            pool,
            user_repository,
            user_service,
        })
    }

    async fn create_database_pool() -> anyhow::Result<PgPool> {
        let database_url = env::var("DATABASE_URL").context(".env中缺少DATABASE_URL")?;
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .context("数据库连接池创建失败！")?;

        sqlx::migrate!()
            .run(&pool)
            .await
            .context("数据库迁移失败")?;
        Ok(pool)
    }
}
