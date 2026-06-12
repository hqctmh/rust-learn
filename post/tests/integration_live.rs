use std::time::Duration;

use elasticsearch::{DeleteParts, Elasticsearch, SearchParts, http::transport::Transport};
use futures_util::StreamExt;
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
#[ignore]
async fn runtime_integration_handler_drains_live_redis_nats_and_elasticsearch() {
    let config = post::state::RuntimeConfig {
        database_url: "postgres://post:post@localhost:5433/post".to_string(),
        redis_url: "redis://localhost:6380".to_string(),
        home_sidebar_cache_enabled: false,
        home_sidebar_cache_ttl_seconds: 60,
        nats_url: "nats://localhost:4222".to_string(),
        rustfs_bucket: "post-assets".to_string(),
        elasticsearch_url: "http://localhost:9200".to_string(),
        elasticsearch_search_index: "posts".to_string(),
        search_backend: "postgres".to_string(),
        integration_worker_enabled: true,
        integration_worker_batch_size: 20,
        integration_worker_max_attempts: 3,
        integration_worker_interval_millis: 100,
    };

    let suffix = Uuid::new_v4().simple().to_string();
    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("connect postgres");
    let redis_client = redis::Client::open(config.redis_url.as_str()).expect("redis client");
    let mut redis_connection = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("connect redis");
    let nats_client = async_nats::connect(config.nats_url.as_str())
        .await
        .expect("connect nats");
    let subject = format!("post.live.{suffix}");
    let mut subscriber = nats_client
        .subscribe(subject.clone())
        .await
        .expect("subscribe nats subject");
    let elasticsearch = Elasticsearch::new(
        Transport::single_node(config.elasticsearch_url.as_str()).expect("elasticsearch transport"),
    );

    let cache_key = format!("post:live:{suffix}");
    redis::cmd("SET")
        .arg(&cache_key)
        .arg("cached")
        .query_async::<()>(&mut redis_connection)
        .await
        .expect("seed redis cache key");
    let document_id = Uuid::new_v4();
    let index = "post-live-e2e";
    let _ = elasticsearch
        .delete(DeleteParts::IndexId(index, &document_id.to_string()))
        .send()
        .await;

    post::repositories::integrations::PostgresIntegrationRepository::insert_actions(
        &pool,
        &[
            post::domain::integrations::IntegrationAction::CacheInvalidate(
                post::domain::integrations::CacheInvalidation {
                    keys: vec![cache_key.clone()],
                    reason: format!("live.cache.{suffix}"),
                },
            ),
            post::domain::integrations::IntegrationAction::NatsPublish(
                post::domain::integrations::IntegrationEvent {
                    subject: subject.clone(),
                    aggregate_id: document_id,
                    payload_summary: format!("live nats {suffix}"),
                },
            ),
            post::domain::integrations::IntegrationAction::SearchIndex(
                post::domain::integrations::SearchIndexMutation::Upsert(
                    post::domain::integrations::SearchIndexDocument {
                        index: index.to_string(),
                        document_id,
                        title: format!("Live integration {suffix}"),
                        summary: "Redis NATS Elasticsearch live e2e".to_string(),
                        body: format!("body marker {suffix}"),
                        category_name: Some("测试".to_string()),
                        tags: vec!["live".to_string(), "integration".to_string()],
                        author_id: Uuid::new_v4(),
                    },
                ),
            ),
        ],
    )
    .await
    .expect("insert live outbox rows");

    let handler = post::integration_handler::RuntimeIntegrationHandler::from_config(&config)
        .await
        .expect("create runtime integration handler");
    let worker = post::integration_worker::IntegrationOutboxWorker::new(handler, 20, 3);
    let report = worker.drain_once(&pool).await.expect("drain live outbox");

    assert!(report.scanned >= 3);
    assert!(report.processed >= 3);
    assert_eq!(report.failed, 0);

    let redis_exists: bool = redis::cmd("EXISTS")
        .arg(&cache_key)
        .query_async(&mut redis_connection)
        .await
        .expect("check redis cache key");
    assert!(
        !redis_exists,
        "cache key should be deleted by Redis handler"
    );

    let message = tokio::time::timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("wait for nats message")
        .expect("nats message should arrive");
    let payload = String::from_utf8(message.payload.to_vec()).expect("nats payload utf8");
    assert!(payload.contains(&suffix));

    let search_body = wait_for_search_hit(&elasticsearch, index, &suffix).await;
    assert_eq!(
        search_body["hits"]["hits"][0]["_source"]["title"].as_str(),
        Some(format!("Live integration {suffix}").as_str())
    );

    post::repositories::integrations::PostgresIntegrationRepository::insert_actions(
        &pool,
        &[post::domain::integrations::IntegrationAction::SearchIndex(
            post::domain::integrations::SearchIndexMutation::Delete {
                index: index.to_string(),
                document_id,
            },
        )],
    )
    .await
    .expect("insert delete outbox row");

    let handler = post::integration_handler::RuntimeIntegrationHandler::from_config(&config)
        .await
        .expect("create delete handler");
    let worker = post::integration_worker::IntegrationOutboxWorker::new(handler, 20, 3);
    worker
        .drain_once(&pool)
        .await
        .expect("drain delete outbox row");

    let _ = elasticsearch
        .delete(DeleteParts::IndexId(index, &document_id.to_string()))
        .send()
        .await;
}

#[tokio::test]
#[ignore]
async fn rustfs_object_store_uploads_to_live_rustfs() {
    let suffix = Uuid::new_v4().simple().to_string();
    let store = post::object_store::RustfsObjectStore::from_config(
        post::object_store::RustfsObjectStoreConfig {
            bucket: "post-assets".to_string(),
        },
    )
    .await
    .expect("create RustFS object store");
    let object = post::domain::files::FileBinaryUploadRequest {
        original_filename: format!("rustfs-live-{suffix}.png"),
        mime_type: "image/png".to_string(),
        content_base64: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("post rustfs live {suffix}").as_bytes(),
        ),
        usage: post::domain::files::FileUsage::MarkdownImage,
    }
    .to_object_upload()
    .expect("build upload object");

    store
        .put_object(object)
        .await
        .expect("upload object to live RustFS");
}

async fn wait_for_search_hit(
    client: &Elasticsearch,
    index: &str,
    marker: &str,
) -> serde_json::Value {
    for _ in 0..10 {
        let response = client
            .search(SearchParts::Index(&[index]))
            .body(json!({
                "query": {
                    "multi_match": {
                        "query": marker,
                        "fields": ["title", "summary", "body", "tags"]
                    }
                }
            }))
            .send()
            .await
            .expect("search elasticsearch");
        let body: Value = response.json().await.expect("read search response");
        if body["hits"]["total"]["value"].as_i64().unwrap_or_default() > 0 {
            return body;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    panic!("Elasticsearch document was not searchable for marker {marker}");
}
