use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    auth::SessionUser,
    user_admin::{AdminUserRow, AuditContext, AuditLogEntry},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminUserDbRow {
    pub user_id: Uuid,
    pub username: String,
    pub nickname: String,
    pub roles: Vec<String>,
    pub disabled: bool,
    pub post_count: i64,
    pub comment_count: i64,
}

impl From<AdminUserDbRow> for AdminUserRow {
    fn from(row: AdminUserDbRow) -> Self {
        Self {
            user_id: row.user_id,
            username: row.username,
            nickname: row.nickname,
            roles: row.roles,
            disabled: row.disabled,
            post_count: row.post_count.max(0) as usize,
            comment_count: row.comment_count.max(0) as usize,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditLogDbRow {
    pub audit_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub target_label: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
}

impl From<AuditLogDbRow> for AuditLogEntry {
    fn from(row: AuditLogDbRow) -> Self {
        let target_id = row.target_id.unwrap_or_else(Uuid::nil);
        Self {
            audit_id: row.audit_id,
            actor_id: row.actor_id.unwrap_or_else(Uuid::nil),
            actor_name: row.actor_name.unwrap_or_else(|| "系统".to_string()),
            action: row.action,
            target_type: row.target_type,
            target_id,
            target_label: row
                .target_label
                .unwrap_or_else(|| target_id.hyphenated().to_string()),
            before: row.before,
            after: row.after,
            ip: row.ip,
            user_agent: row.user_agent,
            created_at: row.created_at,
        }
    }
}

pub struct PostgresUserAdminRepository;

impl PostgresUserAdminRepository {
    pub async fn list_users(pool: &sqlx::PgPool) -> sqlx::Result<Vec<AdminUserRow>> {
        let rows = sqlx::query_as!(
            AdminUserDbRow,
            r#"
select
    u.user_id,
    u.username,
    u.nickname,
    coalesce(
        nullif(array_remove(array_agg(distinct r.code order by r.code), null), array[]::text[]),
        array['member']::text[]
    ) as "roles!: Vec<String>",
    (u.status = 'disabled') as "disabled!",
    count(distinct p.post_id) as "post_count!",
    count(distinct c.comment_id) as "comment_count!"
from users u
left join user_roles ur on ur.user_id = u.user_id
left join roles r on r.role_id = ur.role_id
left join posts p on p.author_id = u.user_id and p.status <> 'deleted'
left join comments c on c.author_id = u.user_id
group by u.user_id, u.username, u.nickname, u.status
order by u.username asc
"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(AdminUserRow::from).collect())
    }

    pub async fn find_user(
        pool: &sqlx::PgPool,
        user_id: Uuid,
    ) -> sqlx::Result<Option<AdminUserRow>> {
        let row = sqlx::query_as!(
            AdminUserDbRow,
            r#"
select
    u.user_id,
    u.username,
    u.nickname,
    coalesce(
        nullif(array_remove(array_agg(distinct r.code order by r.code), null), array[]::text[]),
        array['member']::text[]
    ) as "roles!: Vec<String>",
    (u.status = 'disabled') as "disabled!",
    count(distinct p.post_id) as "post_count!",
    count(distinct c.comment_id) as "comment_count!"
from users u
left join user_roles ur on ur.user_id = u.user_id
left join roles r on r.role_id = ur.role_id
left join posts p on p.author_id = u.user_id and p.status <> 'deleted'
left join comments c on c.author_id = u.user_id
where u.user_id = $1
group by u.user_id, u.username, u.nickname, u.status
limit 1
"#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(AdminUserRow::from))
    }

    pub async fn set_user_disabled(
        pool: &sqlx::PgPool,
        actor: &SessionUser,
        user_id: Uuid,
        disabled: bool,
        context: AuditContext,
    ) -> sqlx::Result<Option<AdminUserRow>> {
        let before = Self::find_user(pool, user_id).await?;
        if before.is_none() {
            return Ok(None);
        }

        let status = if disabled { "disabled" } else { "active" };
        let rows_affected = sqlx::query!(
            r#"
update users
set status = $2,
    updated_at = now()
where user_id = $1
"#,
            user_id,
            status
        )
        .execute(pool)
        .await?
        .rows_affected();
        if rows_affected == 0 {
            return Ok(None);
        }

        let after = Self::find_user(pool, user_id).await?;
        if let (Some(before), Some(after)) = (&before, &after) {
            Self::insert_audit_log(
                pool,
                actor,
                if disabled {
                    "user.disable"
                } else {
                    "user.enable"
                },
                "user",
                user_id,
                Some(snapshot(before)),
                Some(snapshot(after)),
                context,
            )
            .await?;
        }

        Ok(after)
    }

    pub async fn update_user_roles(
        pool: &sqlx::PgPool,
        actor: &SessionUser,
        user_id: Uuid,
        roles: Vec<String>,
        context: AuditContext,
    ) -> sqlx::Result<Option<AdminUserRow>> {
        let before = Self::find_user(pool, user_id).await?;
        if before.is_none() {
            return Ok(None);
        }

        let mut tx = pool.begin().await?;
        sqlx::query!(
            r#"
delete from user_roles
where user_id = $1
"#,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        for role in &roles {
            sqlx::query!(
                r#"
insert into roles (role_id, code, name)
values ($1, $2, $3)
on conflict (code) do nothing
"#,
                Uuid::new_v4(),
                role,
                role
            )
            .execute(&mut *tx)
            .await?;

            sqlx::query!(
                r#"
insert into user_roles (user_id, role_id)
select $1, role_id
from roles
where code = $2
on conflict (user_id, role_id) do nothing
"#,
                user_id,
                role
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        let after = Self::find_user(pool, user_id).await?;
        if let (Some(before), Some(after)) = (&before, &after) {
            Self::insert_audit_log(
                pool,
                actor,
                "user.roles.update",
                "user",
                user_id,
                Some(snapshot(before)),
                Some(snapshot(after)),
                context,
            )
            .await?;
        }

        Ok(after)
    }

    pub async fn list_audit_logs(pool: &sqlx::PgPool) -> sqlx::Result<Vec<AuditLogEntry>> {
        let rows = sqlx::query_as!(
            AuditLogDbRow,
            r#"
select
    a.audit_log_id as audit_id,
    a.operator_id as "actor_id?",
    actor.nickname as "actor_name?",
    a.action,
    a.target_type,
    a.target_id as "target_id?",
    target.nickname as "target_label?",
    a.before_data #>> '{}' as "before?",
    a.after_data #>> '{}' as "after?",
    a.ip_address as "ip?",
    a.user_agent as "user_agent?",
    a.created_at
from audit_logs a
left join users actor on actor.user_id = a.operator_id
left join users target on target.user_id = a.target_id and a.target_type = 'user'
order by a.created_at desc
"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(AuditLogEntry::from).collect())
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_audit_log(
        pool: &sqlx::PgPool,
        actor: &SessionUser,
        action: &str,
        target_type: &str,
        target_id: Uuid,
        before: Option<String>,
        after: Option<String>,
        context: AuditContext,
    ) -> sqlx::Result<()> {
        let before = before.as_deref();
        let after = after.as_deref();
        sqlx::query!(
            r#"
insert into audit_logs (
    audit_log_id,
    operator_id,
    action,
    target_type,
    target_id,
    before_data,
    after_data,
    ip_address,
    user_agent
)
values (
    $1,
    $2,
    $3,
    $4,
    $5,
    case when $6::text is null then null else to_jsonb($6::text) end,
    case when $7::text is null then null else to_jsonb($7::text) end,
    $8,
    $9
)
"#,
            Uuid::new_v4(),
            actor.user_id,
            action,
            target_type,
            target_id,
            before,
            after,
            context.ip,
            context.user_agent
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

fn snapshot(row: &AdminUserRow) -> String {
    format!(
        "username={},disabled={},roles={}",
        row.username,
        row.disabled,
        row.roles.join("|")
    )
}
