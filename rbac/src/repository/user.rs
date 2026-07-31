use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::{
    dto::{Page, UserPageQuery},
    model::User,
};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
        display_name: Option<&str>,
    ) -> anyhow::Result<User> {
        sqlx::query_as!(
            User,
            r#"
                insert into users(username,password_hash,display_name)
                values ($1,$2,$3)
                returning *
            "#,
            username,
            password,
            display_name
        )
        .fetch_one(&self.pool)
        .await
        .context("创建user失败")
    }

    pub async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        sqlx::query_as!(
            User,
            r#"
                select * from users where username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .context("find_by_username查询失败")
    }

    pub async fn select_by_id(&self, id: Uuid) -> anyhow::Result<User> {
        sqlx::query_as!(
            User,
            r#"
                select * from users where id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("select_by_id查询失败")
    }

    pub async fn select_by_page(&self, query: &UserPageQuery) -> anyhow::Result<Page<User>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let mut count_builder =
            QueryBuilder::<Postgres>::new("select count(1) from users where 1=1 ");

        Self::push_user_conditions(&mut count_builder, query);

        let total = count_builder
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await?;

        let mut data_builder = QueryBuilder::<Postgres>::new(
            r#"
                select * from users where 1=1
            "#,
        );

        Self::push_user_conditions(&mut data_builder, query);

        data_builder
            .push(" order by created_at desc, id desc ")
            .push(" limit ")
            .push_bind(page_size)
            .push(" offset ")
            .push_bind(offset);

        let items = data_builder
            .build_query_as::<User>()
            .fetch_all(&self.pool)
            .await?;

        Ok(Page {
            items,
            total,
            page,
            page_size,
        })
    }

    fn push_user_conditions(builder: &mut QueryBuilder<Postgres>, query: &UserPageQuery) {
        if let Some(username) = query.username.as_deref() {
            builder
                .push(" and username ilike ")
                .push_bind(format!("%{username}%"));
        }

        if let Some(display_name) = query.display_name.as_deref() {
            builder
                .push(" and display_name ilike ")
                .push_bind(format!("%{display_name}%"));
        }

        if let Some(is_active) = query.is_active {
            builder.push(" and is_active = ").push_bind(is_active);
        }
    }
}
