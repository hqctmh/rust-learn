use aws_credential_types::Credentials;
use aws_sdk_s3::{Client, config::Region, primitives::ByteStream};

use crate::{domain::files::FileObjectUpload, state::RuntimeConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustfsObjectStoreConfig {
    pub endpoint_url: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: bool,
}

impl RustfsObjectStoreConfig {
    pub fn from_runtime_config(config: &RuntimeConfig) -> Self {
        Self {
            endpoint_url: config.rustfs_endpoint.clone(),
            region: config.rustfs_region.clone(),
            bucket: config.rustfs_bucket.clone(),
            access_key_id: config.rustfs_access_key.clone(),
            secret_access_key: config.rustfs_secret_key.clone(),
            force_path_style: config.rustfs_force_path_style,
        }
    }
}

pub struct RustfsObjectStore {
    client: Client,
    bucket: String,
}

impl RustfsObjectStore {
    pub async fn from_config(config: RustfsObjectStoreConfig) -> Result<Self, String> {
        let credentials = Credentials::new(
            config.access_key_id,
            config.secret_access_key,
            None,
            None,
            "rustfs",
        );
        let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(config.region))
            .credentials_provider(credentials)
            .endpoint_url(config.endpoint_url)
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
            .force_path_style(config.force_path_style)
            .build();

        Ok(Self {
            client: Client::from_conf(s3_config),
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
            .map_err(|error| format!("RustFS 对象上传失败: {error}"))?;

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
            .map_err(|error| format!("RustFS bucket 创建失败: {error}"))?;

        Ok(())
    }
}
