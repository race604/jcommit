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
    model: String,
}

const SYSTEM_PROMPT: &str = "
    You are a Git Commit Message Generator. 
    Based on the provided Git diff content, generate a concise, clear commit message that follows the Conventional Commits specification.
    If additional hints are provided by the user, take them into consideration as well.
    Please do not output the body of commit message unless user request you to do so.
    Some output example:
    
    Ouput examples witout body:
    * feat: allow provided config object to extend other configs
    * feat(api): send an email to the customer when a product is shipped
    
    Output example with body:
    fix: prevent racing of requests

    - Introduce a request id and a reference to latest request. Dismiss
    incoming responses other than from latest request.

    - Remove timeouts which were used to mitigate the racing issue but are
    obsolete now.";

const DEFAULT_API_ENDPOINT: &str = "https://api.openai.com/v1";

fn ensure_chat_completions_endpoint(endpoint: &str) -> String {
    if !endpoint.ends_with("/chat/completions") {
        format!("{}/chat/completions", endpoint.trim_end_matches('/'))
    } else {
        endpoint.to_string()
    }
}
const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

impl AiService {
    pub fn new(api_endpoint: Option<String>, model: Option<String>, api_key: Option<String>) -> Self {
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

        let api_endpoint = api_endpoint.unwrap_or_else(|| DEFAULT_API_ENDPOINT.to_string());
        let api_endpoint = ensure_chat_completions_endpoint(&api_endpoint);

        Self {
            client,
            api_endpoint,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
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