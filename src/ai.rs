use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AiRequest {
    diff: String,
    message: Option<String>,
}

#[derive(Deserialize)]
struct AiResponse {
    commit_message: String,
}

pub struct AiService {
    client: reqwest::Client,
    api_endpoint: String,
}

impl AiService {
    pub fn new(api_endpoint: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_endpoint,
        }
    }

    pub async fn generate_commit_message(
        &self,
        diff: String,
        message: Option<String>,
    ) -> Result<String> {
        let request = AiRequest { diff, message };
        let response = self
            .client
            .post(&self.api_endpoint)
            .json(&request)
            .send()
            .await?
            .json::<AiResponse>()
            .await?;

        Ok(response.commit_message)
    }
}