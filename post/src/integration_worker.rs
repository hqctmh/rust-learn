use std::{future::Future, pin::Pin};

use crate::repositories::integrations::{IntegrationOutboxRow, PostgresIntegrationRepository};

pub trait IntegrationActionHandler {
    fn handle<'a>(
        &'a self,
        row: &'a IntegrationOutboxRow,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegrationDrainReport {
    pub scanned: usize,
    pub processed: usize,
    pub failed: usize,
}

pub struct IntegrationOutboxWorker<H> {
    handler: H,
    batch_size: i64,
    max_attempts: i32,
}

impl<H> IntegrationOutboxWorker<H>
where
    H: IntegrationActionHandler + Sync,
{
    pub fn new(handler: H, batch_size: i64, max_attempts: i32) -> Self {
        Self {
            handler,
            batch_size,
            max_attempts,
        }
    }

    pub async fn drain_once(&self, pool: &sqlx::PgPool) -> sqlx::Result<IntegrationDrainReport> {
        let rows = PostgresIntegrationRepository::list_pending(pool, self.batch_size).await?;
        let scanned = rows.len();
        let mut processed = 0;
        let mut failed = 0;

        for row in rows {
            match self.handler.handle(&row).await {
                Ok(()) => {
                    PostgresIntegrationRepository::mark_processed(pool, row.outbox_id).await?;
                    processed += 1;
                }
                Err(error) => {
                    PostgresIntegrationRepository::mark_failed(
                        pool,
                        row.outbox_id,
                        &error,
                        self.max_attempts,
                    )
                    .await?;
                    failed += 1;
                }
            }
        }

        Ok(IntegrationDrainReport {
            scanned,
            processed,
            failed,
        })
    }
}
