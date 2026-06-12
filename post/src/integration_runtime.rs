use std::time::Duration;

use leptos::logging::{error, log};

use crate::{
    integration_handler::RuntimeIntegrationHandler, integration_worker::IntegrationOutboxWorker,
    state::RuntimeConfig,
};

pub fn spawn_integration_outbox_worker(
    pool: Option<sqlx::PgPool>,
    config: RuntimeConfig,
) -> Option<tokio::task::JoinHandle<()>> {
    if !config.integration_worker_enabled {
        log!("integration worker disabled");
        return None;
    }
    let Some(pool) = pool else {
        error!("integration worker enabled but DATABASE_URL pool is unavailable");
        return None;
    };

    Some(tokio::spawn(async move {
        let handler = match RuntimeIntegrationHandler::from_config(&config).await {
            Ok(handler) => handler,
            Err(message) => {
                error!("{message}");
                return;
            }
        };
        let worker = IntegrationOutboxWorker::new(
            handler,
            config.integration_worker_batch_size,
            config.integration_worker_max_attempts,
        );
        let mut interval = tokio::time::interval(Duration::from_millis(
            config.integration_worker_interval_millis,
        ));

        loop {
            interval.tick().await;
            match worker.drain_once(&pool).await {
                Ok(report) if report.scanned > 0 => log!(
                    "integration worker drained scanned={} processed={} failed={}",
                    report.scanned,
                    report.processed,
                    report.failed
                ),
                Ok(_) => {}
                Err(error) => error!("integration worker drain failed: {error}"),
            }
        }
    }))
}
