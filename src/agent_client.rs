use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct AgentClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl AgentClient {
    pub fn new(client_type: &str, api_key: &str, model: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        
        let (base_url, default_model) = match client_type {
            "claude" => (
                "https://api.anthropic.com/v1/messages",
                "claude-3-5-sonnet-20241022",
            ),
            "openai" => (
                "https://api.openai.com/v1/chat/completions",
                "gpt-4o",
            ),
            "gemini" => (
                "https://generativelanguage.googleapis.com/v1beta/models",
                "gemini-2.0-flash-exp",
            ),
            "groq" => (
                "https://api.groq.com/openai/v1/chat/completions",
                "llama3-70b-8192",
            ),
            _ => return Err(format!("Unknown client type: {}", client_type).into()),
        };
        
        Ok(AgentClient {
            client,
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            model: model.unwrap_or(default_model).to_string(),
        })
    }
    
    pub async fn send_message(&self, message: &str) -> Result<AgentResponse, Box<dyn std::error::Error>> {
        match self.base_url.contains("anthropic") {
            true => self.send_claude(message).await,
            false if self.base_url.contains("generativelanguage") => self.send_gemini(message).await,
            _ => self.send_openai_compatible(message).await,
        }
    }
    
    async fn send_claude(&self, message: &str) -> Result<AgentResponse, Box<dyn std::error::Error>> {
        #[derive(Serialize)]
        struct ClaudeRequest {
            model: String,
            max_tokens: u32,
            messages: Vec<Message>,
        }
        
        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }
        
        #[derive(Deserialize)]
        struct ClaudeResponse {
            content: Vec<ContentBlock>,
            usage: Usage,
        }
        
        #[derive(Deserialize)]
        struct ContentBlock {
            text: String,
        }
        
        #[derive(Deserialize)]
        struct Usage {
            input_tokens: u32,
            output_tokens: u32,
        }
        
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".into(),
                content: message.into(),
            }],
        };
        
        let start = std::time::Instant::now();
        
        let response = self.client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?
            .json::<ClaudeResponse>()
            .await?;
        
        let latency = start.elapsed().as_millis() as u64;
        
        Ok(AgentResponse {
            content: response.content.first().map(|c| c.text.clone()).unwrap_or_default(),
            model: self.model.clone(),
            tokens_used: response.usage.input_tokens + response.usage.output_tokens,
            latency_ms: latency,
        })
    }
    
    async fn send_openai_compatible(&self, message: &str) -> Result<AgentResponse, Box<dyn std::error::Error>> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<Message>,
            max_tokens: u32,
        }
        
        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }
        
        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<Choice>,
            usage: Usage,
        }
        
        #[derive(Deserialize)]
        struct Choice {
            message: MessageResponse,
        }
        
        #[derive(Deserialize)]
        struct MessageResponse {
            content: String,
        }
        
        #[derive(Deserialize)]
        struct Usage {
            total_tokens: u32,
        }
        
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".into(),
                content: message.into(),
            }],
            max_tokens: 4096,
        };
        
        let start = std::time::Instant::now();
        
        let response = self.client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<OpenAIResponse>()
            .await?;
        
        let latency = start.elapsed().as_millis() as u64;
        
        let content = response.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        
        Ok(AgentResponse {
            content,
            model: self.model.clone(),
            tokens_used: response.usage.total_tokens,
            latency_ms: latency,
        })
    }
    
    async fn send_gemini(&self, message: &str) -> Result<AgentResponse, Box<dyn std::error::Error>> {
        #[derive(Serialize)]
        struct GeminiRequest {
            contents: Vec<Content>,
        }
        
        #[derive(Serialize)]
        struct Content {
            parts: Vec<Part>,
        }
        
        #[derive(Serialize)]
        struct Part {
            text: String,
        }
        
        #[derive(Deserialize)]
        struct GeminiResponse {
            candidates: Vec<Candidate>,
        }
        
        #[derive(Deserialize)]
        struct Candidate {
            content: ContentResponse,
        }
        
        #[derive(Deserialize)]
        struct ContentResponse {
            parts: Vec<PartResponse>,
        }
        
        #[derive(Deserialize)]
        struct PartResponse {
            text: String,
        }
        
        let url = format!("{}//{}:generateContent?key={}", self.base_url, self.model, self.api_key);
        
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: message.into(),
                }],
            }],
        };
        
        let start = std::time::Instant::now();
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;
        
        let latency = start.elapsed().as_millis() as u64;
        
        let content = response.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();
        
        Ok(AgentResponse {
            content,
            model: self.model.clone(),
            tokens_used: 0, // Gemini doesn't return token count in this endpoint
            latency_ms: latency,
        })
    }
}

pub fn create_client(client_type: &str) -> Result<AgentClient, Box<dyn std::error::Error>> {
    let env_key = match client_type {
        "claude" => "ANTHROPIC_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "gemini" => "GEMINI_API_KEY",
        "groq" => "GROQ_API_KEY",
        _ => return Err("Unknown client type".into()),
    };
    
    let api_key = std::env::var(env_key)
        .map_err(|_| format!("{} environment variable not set", env_key))?;
    
    AgentClient::new(client_type, &api_key, None)
}
