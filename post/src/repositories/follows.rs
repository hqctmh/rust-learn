use uuid::Uuid;

use crate::domain::reactions::FollowState;

pub struct PostgresFollowRepository;

impl PostgresFollowRepository {
    pub async fn toggle_follow(
        pool: &sqlx::PgPool,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> sqlx::Result<FollowState> {
        let mut tx = pool.begin().await?;
        let deleted = sqlx::query!(
            r#"
delete from follows
where follower_id = $1
  and followee_id = $2
returning follower_id
"#,
            follower_id,
            followee_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let following = deleted.is_none();
        if following {
            sqlx::query!(
                r#"
insert into follows (
    follower_id,
    followee_id
)
values ($1, $2)
on conflict do nothing
"#,
                follower_id,
                followee_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(FollowState {
            follower_id,
            followee_id,
            following,
        })
    }
}
