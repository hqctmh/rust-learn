use uuid::Uuid;

use crate::domain::reactions::ToggleResult;

pub struct PostgresReactionRepository;

impl PostgresReactionRepository {
    pub async fn toggle_post_like(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        post_id: Uuid,
    ) -> sqlx::Result<ToggleResult> {
        let mut tx = pool.begin().await?;
        let deleted = sqlx::query!(
            r#"
delete from post_likes
where post_id = $1
  and user_id = $2
returning user_id
"#,
            post_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let active = deleted.is_none();
        if active {
            sqlx::query!(
                r#"
insert into post_likes (
    post_id,
    user_id
)
values ($1, $2)
on conflict do nothing
"#,
                post_id,
                user_id
            )
            .execute(&mut *tx)
            .await?;
        }

        let count = if active {
            sqlx::query!(
                r#"
update posts
set like_count = like_count + 1,
    updated_at = now()
where post_id = $1
returning like_count as "count!"
"#,
                post_id
            )
            .fetch_one(&mut *tx)
            .await?
            .count
        } else {
            sqlx::query!(
                r#"
update posts
set like_count = greatest(like_count - 1, 0::bigint),
    updated_at = now()
where post_id = $1
returning like_count as "count!"
"#,
                post_id
            )
            .fetch_one(&mut *tx)
            .await?
            .count
        };

        tx.commit().await?;
        Ok(ToggleResult { active, count })
    }

    pub async fn toggle_post_favorite(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        post_id: Uuid,
    ) -> sqlx::Result<ToggleResult> {
        let mut tx = pool.begin().await?;
        let deleted = sqlx::query!(
            r#"
delete from post_favorites
where post_id = $1
  and user_id = $2
returning user_id
"#,
            post_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let active = deleted.is_none();
        if active {
            sqlx::query!(
                r#"
insert into post_favorites (
    post_id,
    user_id
)
values ($1, $2)
on conflict do nothing
"#,
                post_id,
                user_id
            )
            .execute(&mut *tx)
            .await?;
        }

        let count = if active {
            sqlx::query!(
                r#"
update posts
set favorite_count = favorite_count + 1,
    updated_at = now()
where post_id = $1
returning favorite_count as "count!"
"#,
                post_id
            )
            .fetch_one(&mut *tx)
            .await?
            .count
        } else {
            sqlx::query!(
                r#"
update posts
set favorite_count = greatest(favorite_count - 1, 0::bigint),
    updated_at = now()
where post_id = $1
returning favorite_count as "count!"
"#,
                post_id
            )
            .fetch_one(&mut *tx)
            .await?
            .count
        };

        tx.commit().await?;
        Ok(ToggleResult { active, count })
    }
}
