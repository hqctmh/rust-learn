use std::collections::HashMap;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{comments::CommentNode, reactions::ToggleResult};

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct CommentRow {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub author_id: Uuid,
    pub author_name: String,
    pub content: String,
    pub status: String,
    pub like_count: i64,
    pub created_at: OffsetDateTime,
    pub post_author_id: Uuid,
}

impl CommentRow {
    fn into_node(self) -> CommentNode {
        CommentNode {
            comment_id: self.comment_id,
            post_id: self.post_id,
            parent_comment_id: self.parent_comment_id,
            author_id: self.author_id,
            author_name: self.author_name,
            content: self.content,
            deleted: self.status == "deleted",
            author_reply: self.author_id == self.post_author_id,
            like_count: self.like_count,
            created_at: self.created_at,
            replies: Vec::new(),
        }
    }
}

pub struct PostgresCommentRepository;

impl PostgresCommentRepository {
    pub fn comments_for_post_sql() -> &'static str {
        r#"
select
    c.comment_id,
    c.post_id,
    c.parent_comment_id,
    c.author_id,
    u.nickname as author_name,
    c.content,
    c.status,
    c.like_count,
    c.created_at,
    p.author_id as post_author_id
from comments c
join users u on u.user_id = c.author_id
join posts p on p.post_id = c.post_id
where c.post_id = $1
order by c.created_at asc
"#
    }

    pub fn build_comment_tree(rows: Vec<CommentRow>) -> Vec<CommentNode> {
        let mut by_parent: HashMap<Option<Uuid>, Vec<CommentNode>> = HashMap::new();
        for row in rows {
            let node = row.into_node();
            by_parent
                .entry(node.parent_comment_id)
                .or_default()
                .push(node);
        }

        attach_replies(None, &mut by_parent)
            .into_iter()
            .map(mask_deleted)
            .collect()
    }

    pub async fn list_for_post(
        pool: &sqlx::PgPool,
        post_id: Uuid,
    ) -> sqlx::Result<Vec<CommentNode>> {
        let rows = sqlx::query_as!(
            CommentRow,
            r#"
select
    c.comment_id,
    c.post_id,
    c.parent_comment_id,
    c.author_id,
    u.nickname as author_name,
    c.content,
    c.status,
    c.like_count,
    c.created_at,
    p.author_id as post_author_id
from comments c
join users u on u.user_id = c.author_id
join posts p on p.post_id = c.post_id
where c.post_id = $1
order by c.created_at asc
"#,
            post_id
        )
        .fetch_all(pool)
        .await?;

        Ok(Self::build_comment_tree(rows))
    }

    pub async fn insert_comment(pool: &sqlx::PgPool, comment: &CommentNode) -> sqlx::Result<()> {
        let mut tx = pool.begin().await?;
        sqlx::query!(
            r#"
insert into comments (
    comment_id,
    post_id,
    parent_comment_id,
    author_id,
    content,
    status,
    like_count,
    created_at
)
values ($1, $2, $3, $4, $5, 'visible', $6, $7)
"#,
            comment.comment_id,
            comment.post_id,
            comment.parent_comment_id,
            comment.author_id,
            comment.content,
            comment.like_count,
            comment.created_at
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
update posts
set comment_count = comment_count + 1,
    updated_at = now()
where post_id = $1
"#,
            comment.post_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn find_by_id(
        pool: &sqlx::PgPool,
        comment_id: Uuid,
    ) -> sqlx::Result<Option<CommentNode>> {
        let row = sqlx::query_as!(
            CommentRow,
            r#"
select
    c.comment_id,
    c.post_id,
    c.parent_comment_id,
    c.author_id,
    u.nickname as author_name,
    c.content,
    c.status,
    c.like_count,
    c.created_at,
    p.author_id as post_author_id
from comments c
join users u on u.user_id = c.author_id
join posts p on p.post_id = c.post_id
where c.comment_id = $1
limit 1
"#,
            comment_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(CommentRow::into_node).map(mask_deleted))
    }

    pub async fn mark_deleted(pool: &sqlx::PgPool, comment: &CommentNode) -> sqlx::Result<()> {
        let mut tx = pool.begin().await?;
        if !comment.deleted {
            sqlx::query!(
                r#"
update comments
set status = 'deleted',
    updated_at = now()
where comment_id = $1
"#,
                comment.comment_id
            )
            .execute(&mut *tx)
            .await?;

            sqlx::query!(
                r#"
update posts
set comment_count = greatest(comment_count - 1, 0::bigint),
    updated_at = now()
where post_id = $1
"#,
                comment.post_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await
    }

    pub async fn toggle_like(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> sqlx::Result<ToggleResult> {
        let mut tx = pool.begin().await?;
        let deleted = sqlx::query!(
            r#"
delete from comment_likes
where comment_id = $1
  and user_id = $2
returning user_id
"#,
            comment_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let active = deleted.is_none();
        if active {
            sqlx::query!(
                r#"
insert into comment_likes (
    comment_id,
    user_id
)
values ($1, $2)
on conflict do nothing
"#,
                comment_id,
                user_id
            )
            .execute(&mut *tx)
            .await?;
        }

        let count = if active {
            sqlx::query!(
                r#"
update comments
set like_count = like_count + 1,
    updated_at = now()
where comment_id = $1
returning like_count as "count!"
"#,
                comment_id
            )
            .fetch_one(&mut *tx)
            .await?
            .count
        } else {
            sqlx::query!(
                r#"
update comments
set like_count = greatest(like_count - 1, 0::bigint),
    updated_at = now()
where comment_id = $1
returning like_count as "count!"
"#,
                comment_id
            )
            .fetch_one(&mut *tx)
            .await?
            .count
        };

        tx.commit().await?;
        Ok(ToggleResult { active, count })
    }
}

fn attach_replies(
    parent_comment_id: Option<Uuid>,
    by_parent: &mut HashMap<Option<Uuid>, Vec<CommentNode>>,
) -> Vec<CommentNode> {
    let mut nodes = by_parent.remove(&parent_comment_id).unwrap_or_default();
    for node in &mut nodes {
        node.replies = attach_replies(Some(node.comment_id), by_parent);
    }
    nodes
}

fn mask_deleted(mut comment: CommentNode) -> CommentNode {
    if comment.deleted {
        comment.content = "该评论已被删除".to_string();
    }
    comment.replies = comment.replies.into_iter().map(mask_deleted).collect();
    comment
}
