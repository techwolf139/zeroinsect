use reqwest::Client;
use serde::Deserialize;

use super::prompts::{PROMPT_INTENT_PARSING, PROMPT_SCHEMA_MATCHING};
use super::types::{CommandResult, Intent};

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct SchemaMatch {
    #[serde(rename = "compatible")]
    pub compatible: bool,
    #[serde(rename = "confidence")]
    pub confidence: f32,
    #[serde(rename = "field_mappings")]
    pub field_mappings: Vec<FieldMapping>,
    #[serde(rename = "issues")]
    pub issues: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FieldMapping {
    #[serde(rename = "ros1")]
    pub ros1: String,
    #[serde(rename = "ros2")]
    pub ros2: String,
    #[serde(rename = "type_convert")]
    pub type_convert: Option<String>,
}

pub struct LlmEngine {
    client: Client,
    ollama_url: String,
    model: String,
}

impl LlmEngine {
    pub fn new(ollama_url: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            ollama_url: ollama_url.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn parse_intent(&self, command: &str) -> Result<Intent, Box<dyn std::error::Error>> {
        let prompt = format!("{}\n\nCommand: {}", PROMPT_INTENT_PARSING, command);

        let response = self
            .client
            .post(format!("{}/api/generate", self.ollama_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "stream": false
            }))
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(serde_json::from_str(&response.response)?)
    }

    pub async fn match_schemas(
        &self,
        ros1_msg: &str,
        ros2_msg: &str,
    ) -> Result<SchemaMatch, Box<dyn std::error::Error>> {
        let prompt = format!(
            "{}\n\nROS1 Message:\n```\n{}\n```\n\nROS2 Message:\n```\n{}\n```",
            PROMPT_SCHEMA_MATCHING, ros1_msg, ros2_msg
        );

        let response = self
            .client
            .post(format!("{}/api/generate", self.ollama_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "stream": false
            }))
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(serde_json::from_str(&response.response)?)
    }
}
