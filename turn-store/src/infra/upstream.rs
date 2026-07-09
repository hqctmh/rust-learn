use reqwest::{Client, Response, header::ACCEPT};

use crate::domain::model::UpstreamSpeed;

#[derive(Clone)]
pub struct UpstreamClient {
    client: Client,
    url: String,
}

impl UpstreamClient {
    pub fn new(client: Client, url: impl Into<String>) -> Self {
        Self {
            client,
            url: url.into(),
        }
    }

    pub async fn connect(&self, speed: UpstreamSpeed) -> reqwest::Result<Response> {
        self.client
            .get(&self.url)
            .header(ACCEPT, "text/event-stream")
            .query(&[("speed", speed.as_str())])
            .send()
            .await?
            .error_for_status()
    }
}
