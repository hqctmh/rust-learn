use std::collections::HashMap;

use uuid::Uuid;

use crate::domain::{
    auth::SessionUser,
    rbac::{Permission, Role, admin_permissions},
    user_admin::AuditContext,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleDbRow {
    pub code: String,
    pub name: String,
    pub permission_codes: Vec<String>,
}

impl From<RoleDbRow> for Role {
    fn from(row: RoleDbRow) -> Self {
        let permissions = permissions_from_codes(row.permission_codes);
        Self {
            code: row.code,
            name: row.name,
            permissions,
        }
    }
}

pub struct PostgresRbacRepository;

impl PostgresRbacRepository {
    pub async fn ensure_seed_data(pool: &sqlx::PgPool) -> sqlx::Result<()> {
        let mut tx = pool.begin().await?;
        for permission in admin_permissions() {
            sqlx::query!(
                r#"
insert into permissions (permission_id, code, name)
values ($1, $2, $3)
on conflict (code) do update
set name = excluded.name
"#,
                Uuid::new_v4(),
                &permission.code,
                &permission.name
            )
            .execute(&mut *tx)
            .await?;
        }

        for role in seed_roles() {
            sqlx::query!(
                r#"
insert into roles (role_id, code, name)
values ($1, $2, $3)
on conflict (code) do update
set name = excluded.name
"#,
                Uuid::new_v4(),
                &role.code,
                &role.name
            )
            .execute(&mut *tx)
            .await?;

            for permission in &role.permissions {
                sqlx::query!(
                    r#"
insert into role_permissions (role_id, permission_id)
select r.role_id, p.permission_id
from roles r
join permissions p on p.code = $2
where r.code = $1
on conflict (role_id, permission_id) do nothing
"#,
                    &role.code,
                    &permission.code
                )
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await
    }

    pub async fn list_roles(pool: &sqlx::PgPool) -> sqlx::Result<Vec<Role>> {
        let rows = sqlx::query_as!(
            RoleDbRow,
            r#"
select
    r.code,
    r.name,
    coalesce(array_remove(array_agg(p.code order by p.code), null), array[]::text[]) as "permission_codes!: Vec<String>"
from roles r
left join role_permissions rp on rp.role_id = r.role_id
left join permissions p on p.permission_id = rp.permission_id
group by r.role_id, r.code, r.name
order by r.code asc
"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Role::from).collect())
    }

    pub async fn find_role(pool: &sqlx::PgPool, code: &str) -> sqlx::Result<Option<Role>> {
        let row = sqlx::query_as!(
            RoleDbRow,
            r#"
select
    r.code,
    r.name,
    coalesce(array_remove(array_agg(p.code order by p.code), null), array[]::text[]) as "permission_codes!: Vec<String>"
from roles r
left join role_permissions rp on rp.role_id = r.role_id
left join permissions p on p.permission_id = rp.permission_id
where r.code = $1
group by r.role_id, r.code, r.name
limit 1
"#,
            code
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Role::from))
    }

    pub async fn create_role(
        pool: &sqlx::PgPool,
        actor: &SessionUser,
        role: Role,
        context: AuditContext,
    ) -> sqlx::Result<Option<Role>> {
        let inserted = sqlx::query!(
            r#"
insert into roles (role_id, code, name)
values ($1, $2, $3)
on conflict (code) do nothing
returning code
"#,
            Uuid::new_v4(),
            &role.code,
            &role.name
        )
        .fetch_optional(pool)
        .await?;
        if inserted.is_none() {
            return Ok(None);
        }

        Self::replace_role_permissions(pool, &role).await?;
        Self::insert_audit_log(
            pool,
            actor,
            "role.create",
            Some(snapshot(&role)),
            None,
            context,
        )
        .await?;

        Self::find_role(pool, &role.code).await
    }

    pub async fn update_role(
        pool: &sqlx::PgPool,
        actor: &SessionUser,
        before: Role,
        after: Role,
        context: AuditContext,
    ) -> sqlx::Result<Option<Role>> {
        let rows_affected = sqlx::query!(
            r#"
update roles
set name = $2
where code = $1
"#,
            &after.code,
            &after.name
        )
        .execute(pool)
        .await?
        .rows_affected();
        if rows_affected == 0 {
            return Ok(None);
        }

        Self::replace_role_permissions(pool, &after).await?;
        Self::insert_audit_log(
            pool,
            actor,
            "role.update",
            Some(snapshot(&after)),
            Some(snapshot(&before)),
            context,
        )
        .await?;

        Self::find_role(pool, &after.code).await
    }

    pub async fn role_has_users(pool: &sqlx::PgPool, code: &str) -> sqlx::Result<bool> {
        let row = sqlx::query!(
            r#"
select exists (
    select 1
    from user_roles ur
    join roles r on r.role_id = ur.role_id
    where r.code = $1
) as "assigned!"
"#,
            code
        )
        .fetch_one(pool)
        .await?;

        Ok(row.assigned)
    }

    pub async fn delete_role(
        pool: &sqlx::PgPool,
        actor: &SessionUser,
        role: Role,
        context: AuditContext,
    ) -> sqlx::Result<Option<Role>> {
        let rows_affected = sqlx::query!(
            r#"
delete from roles
where code = $1
"#,
            &role.code
        )
        .execute(pool)
        .await?
        .rows_affected();
        if rows_affected == 0 {
            return Ok(None);
        }

        Self::insert_audit_log(
            pool,
            actor,
            "role.delete",
            None,
            Some(snapshot(&role)),
            context,
        )
        .await?;

        Ok(Some(role))
    }

    async fn replace_role_permissions(pool: &sqlx::PgPool, role: &Role) -> sqlx::Result<()> {
        let mut tx = pool.begin().await?;
        for permission in &role.permissions {
            sqlx::query!(
                r#"
insert into permissions (permission_id, code, name)
values ($1, $2, $3)
on conflict (code) do update
set name = excluded.name
"#,
                Uuid::new_v4(),
                &permission.code,
                &permission.name
            )
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query!(
            r#"
delete from role_permissions
where role_id in (
    select role_id
    from roles
    where code = $1
)
"#,
            &role.code
        )
        .execute(&mut *tx)
        .await?;

        for permission in &role.permissions {
            sqlx::query!(
                r#"
insert into role_permissions (role_id, permission_id)
select r.role_id, p.permission_id
from roles r
join permissions p on p.code = $2
where r.code = $1
on conflict (role_id, permission_id) do nothing
"#,
                &role.code,
                &permission.code
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await
    }

    async fn insert_audit_log(
        pool: &sqlx::PgPool,
        actor: &SessionUser,
        action: &str,
        after: Option<String>,
        before: Option<String>,
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
    'role',
    $4,
    case when $5::text is null then null else to_jsonb($5::text) end,
    case when $6::text is null then null else to_jsonb($6::text) end,
    $7,
    $8
)
"#,
            Uuid::new_v4(),
            actor.user_id,
            action,
            Uuid::nil(),
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

fn permissions_from_codes(codes: Vec<String>) -> Vec<Permission> {
    let names = admin_permissions()
        .into_iter()
        .map(|permission| (permission.code.clone(), permission.name))
        .collect::<HashMap<_, _>>();
    codes
        .into_iter()
        .map(|code| {
            let name = names.get(&code).cloned().unwrap_or_else(|| code.clone());
            Permission::new(code, name)
        })
        .collect()
}

fn seed_roles() -> Vec<Role> {
    vec![
        Role {
            code: "admin".to_string(),
            name: "管理员".to_string(),
            permissions: admin_permissions(),
        },
        Role {
            code: "member".to_string(),
            name: "普通用户".to_string(),
            permissions: permissions_from_codes(vec![
                "post:view".to_string(),
                "comment:view".to_string(),
            ]),
        },
        Role {
            code: "moderator".to_string(),
            name: "内容审核员".to_string(),
            permissions: permissions_from_codes(vec![
                "post:view".to_string(),
                "post:update".to_string(),
                "comment:view".to_string(),
                "comment:delete".to_string(),
                "report:view".to_string(),
            ]),
        },
        Role {
            code: "operator".to_string(),
            name: "运营人员".to_string(),
            permissions: permissions_from_codes(vec![
                "announcement:create".to_string(),
                "announcement:publish".to_string(),
                "category:view".to_string(),
                "tag:view".to_string(),
            ]),
        },
    ]
}

fn snapshot(role: &Role) -> String {
    format!(
        "code={},name={},permissions={}",
        role.code,
        role.name,
        role.permissions
            .iter()
            .map(|permission| permission.code.as_str())
            .collect::<Vec<_>>()
            .join("|")
    )
}
