use aws_sdk_s3::{
    Client,
    config::{Credentials, Region},
    primitives::ByteStream,
};

use crate::{domain::files::FileObjectUpload, state::RuntimeConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustfsObjectStoreConfig {
    pub bucket: String,
}

impl RustfsObjectStoreConfig {
    pub fn from_runtime_config(config: &RuntimeConfig) -> Self {
        Self {
            bucket: config.rustfs_bucket.clone(),
        }
    }
}

pub struct RustfsObjectStore {
    client: Client,
    bucket: String,
}

impl RustfsObjectStore {
    pub async fn from_config(config: RustfsObjectStoreConfig) -> Result<Self, String> {
        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let endpoint_url = std::env::var("AWS_ENDPOINT_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string());
        let access_key_id =
            std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_else(|_| "rustfsadmin".to_string());
        let secret_access_key =
            std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_else(|_| "rustfsadmin".to_string());
        let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(region))
            .endpoint_url(endpoint_url)
            .credentials_provider(Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "post-rustfs",
            ))
            .load()
            .await;

        Ok(Self {
            client: Client::new(&shared_config),
            bucket: config.bucket,
        })
    }

    pub async fn put_object(&self, object: FileObjectUpload) -> Result<(), String> {
        self.ensure_bucket().await?;
        self.client
            .put_object()
            .bucket(self.bucket.as_str())
            .key(object.storage_key)
            .content_type(object.content_type)
            .body(ByteStream::from(object.bytes))
            .send()
            .await
            .map_err(|error| format!("RustFS 对象上传失败: {error:?}"))?;

        Ok(())
    }

    async fn ensure_bucket(&self) -> Result<(), String> {
        if self
            .client
            .head_bucket()
            .bucket(self.bucket.as_str())
            .send()
            .await
            .is_ok()
        {
            return Ok(());
        }

        self.client
            .create_bucket()
            .bucket(self.bucket.as_str())
            .send()
            .await
            .map_err(|error| format!("RustFS bucket 创建失败: {error:?}"))?;

        Ok(())
    }
}
