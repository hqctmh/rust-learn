use uuid::Uuid;

pub struct PostgresAdminAuditRepository;

impl PostgresAdminAuditRepository {
    pub async fn insert_audit_log(
        pool: &sqlx::PgPool,
        operator_id: Uuid,
        action: &str,
        target_type: &str,
        target_id: Uuid,
        after: &str,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
insert into audit_logs (
    audit_log_id,
    operator_id,
    action,
    target_type,
    target_id,
    after_data
)
values (
    $1,
    $2,
    $3,
    $4,
    $5,
    to_jsonb($6::text)
)
"#,
            Uuid::new_v4(),
            operator_id,
            action,
            target_type,
            target_id,
            after
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
