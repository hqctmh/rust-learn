use crate::{domain::events::ForumEvent, error::ForumError};

pub struct NatsEventPublisher {
    #[cfg(feature = "ssr")]
    client: async_nats::Client,
}

impl NatsEventPublisher {
    #[cfg(feature = "ssr")]
    pub async fn connect(url: &str) -> Result<Self, async_nats::ConnectError> {
        let client = async_nats::connect(url).await?;
        Ok(Self { client })
    }

    pub fn payload_json(event: &ForumEvent) -> Result<Vec<u8>, ForumError> {
        serde_json::to_vec(event).map_err(|_| ForumError::Internal)
    }

    #[cfg(feature = "ssr")]
    pub async fn publish(&self, event: &ForumEvent) -> Result<(), ForumError> {
        let payload = Self::payload_json(event)?;
        self.client
            .publish(event.subject(), payload.into())
            .await
            .map_err(|_| ForumError::Internal)
    }
}
