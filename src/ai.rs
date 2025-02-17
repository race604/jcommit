use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

#[derive(Serialize)]
struct AiRequest {
    diff: String,
    message: Option<String>,
}

#[derive(Deserialize)]
struct AiResponse {
    commit_message: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

pub struct AiService {
    client: reqwest::Client,
    api_endpoint: String,
    model: String
}

const SYSTEM_PROMPT: &str = "
    You are a Git Commit Message Generator. 
    Based on the provided Git diff content, generate a concise, clear commit message that follows the Conventional Commits specification.
    If additional hints are provided by the user, take them into consideration as well.
    Please only output one line of commit message. Only output detail body of commit message when the user explicitly asks for it.
    Some output example:
    
    Ouput examples without body by default:
    * feat: allow provided config object to extend other configs
    * feat(api): send an email to the customer when a product is shipped
    
    Output example with body when user asks for body:
    fix: prevent racing of requests

    - Introduce a request id and a reference to latest request. Dismiss
    incoming responses other than from latest request.

    - Remove timeouts which were used to mitigate the racing issue but are
    obsolete now.";

const DEFAULT_API_ENDPOINT: &str = "https://api.openai.com/v1";
const DEFAULT_AZURE_API_VERSION: &str = "2023-05-15";
const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

impl AiService {
    fn build_api_endpoint(base_endpoint: &str, is_azure: bool, model: &str, api_version: Option<String>) -> String {
        let base_endpoint = base_endpoint.trim_end_matches('/');
        
        if is_azure {
            let version = api_version.unwrap_or_else(|| DEFAULT_AZURE_API_VERSION.to_string());
            format!("{}/openai/deployments/{}/chat/completions?api-version={}", base_endpoint, model, version)
        } else {
            format!("{}/chat/completions", base_endpoint)
        }
    }

    pub fn new(api_endpoint: Option<String>, model: Option<String>, api_key: Option<String>, is_azure: bool, api_version: Option<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        if let Some(api_key) = api_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        let base_endpoint = api_endpoint.unwrap_or_else(|| DEFAULT_API_ENDPOINT.to_string());
        let model = model.unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let api_endpoint = Self::build_api_endpoint(&base_endpoint, is_azure, &model, api_version);

        Self {
            client,
            api_endpoint,
            model
        }
    }

    pub async fn generate_commit_message(
        &self,
        diff: String,
        message: Option<String>,
        body: bool,
    ) -> Result<String> {
        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Git diff content: \n{}\n", diff),
            },
        ];

        if let Some(msg) = message {
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!("User input hints: {}\n", msg),
            });
        }

        if body {
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: "Please include a detailed description in the commit message body.".to_string(),
            });
        }

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.7,
        };

        let response = self
            .client
            .post(&self.api_endpoint)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("API request failed: HTTP {} - {}", status, error_text);
        }

        let parsed_response: ChatCompletionResponse = response.json().await
            .map_err(|e| anyhow::anyhow!("Parse API reponse failed: {}", e))?;

        if let Some(choice) = parsed_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            anyhow::bail!("No response from OpenAI API")
        }
    }
}