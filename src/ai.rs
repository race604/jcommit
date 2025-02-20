use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use futures_util::StreamExt;
use std::io::Write;

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
    You are a professional software development assistant. Based on the git diff code changes and any additional information provided by the user, generate a commit message that adheres to the following standards:

    1. **Format Specifications**
    - Format: `<type>: <subject>` (e.g., `fix: handle null pointer exception when user is not logged in`)
    - Allowed types: fix/feat/update/refactor/docs/style/test/chore
    - Use English for English-speaking users and Chinese for Chinese-speaking users.
    - **Start the subject with a lowercase letter** (unless it's a proper noun or an exception).

    2. **Content Requirements**
    - Do not use ending punctuation.
    - Be concise and focus on a single change (ensuer no more than 80 characters).
    - Accurately reflect the essence of the code changes (do not simply repeat the diff content).
    - **User input is the highest priority.** If the user provides any textual clues, prioritize them over the git diff analysis. Correct any typos or grammatical errors in the user input while preserving the intended meaning.

    3. **Processing Logic**
    ▫️ When both git_diff and textual clues are provided:
        1. Correct any typos or grammatical errors in user input while maintaining the original intent.
        2. Parse the technical impact of the code changes to ensure the commit message accurately reflects the changes.
        3. Combine the corrected user input with the technical analysis to generate a precise and professional commit message.

    ▫️ When only git_diff is provided:
        1. Analyze the functional modules affected by the changes.
        2. Identify the type of code issue being resolved.
        3. Infer the business-level impact.

    Respond in the following format (do not include any additional content):
    [Generated commit message]
    [Blank line if body is requested]
    [Generated commit body, if requested]

    Output Example 1(Without body requested):
    fix: add exception handling for non-existent users

    Output Example 2(With Body Requested):
    fix: add exception handling for non-existent users

    - Previously, the system would crash when a non-existent username was provided during login.
    - Added a null check for the `user` object and threw a `UserNotFoundException` to handle this case gracefully.
    - This improves error handling and prevents unexpected system crashes.
    ";

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
    ) -> Result<impl futures_util::Stream<Item = Result<String>>> {
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

        let request = serde_json::json!({
            "model": self.model.clone(),
            "messages": messages,
            "temperature": 0.7,
            "stream": true
        });

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

        let stream = response.bytes_stream();
        Ok(stream.map(|chunk| -> Result<String> {
            let chunk = chunk?;
            let text = String::from_utf8(chunk.to_vec())?;
            let mut result = String::new();
            
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = line.trim_start_matches("data: ").trim();
                    if data == "[DONE]" {
                        continue;
                    }
                    if let Ok(response) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = response["choices"][0]["delta"]["content"].as_str() {
                            result.push_str(content);
                        }
                    }
                }
            }
            
            Ok(result)
        }))
    }
}