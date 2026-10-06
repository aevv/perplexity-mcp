use std::{env, time::Duration};

use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerConfig},
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
    transport::{
        stdio,
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;

const API_URL: &str = "https://api.perplexity.ai/chat/completions";
const MAX_QUESTION_CHARS: usize = 5000;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Model {
    /// Fast and cheap. Use for most questions.
    #[default]
    Sonar,
    /// More detailed answers at several times the cost.
    SonarPro,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AskRequest {
    /// The question to ask Perplexity.
    question: String,
    /// Which model to use. Defaults to `sonar`.
    #[serde(default)]
    model: Model,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

#[derive(Clone)]
struct Perplexity {
    http: reqwest::Client,
    api_key: String,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Perplexity {
    fn new(http: reqwest::Client, api_key: String) -> Self {
        Self {
            http,
            api_key,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Ask Perplexity a question and get a short web-grounded answer. Use for up-to-date facts, versions and documentation.",
        annotations(
            title = "Ask Perplexity",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn ask_perplexity(
        &self,
        Parameters(AskRequest { question, model }): Parameters<AskRequest>,
    ) -> Result<String, String> {
        let len = question.chars().count();
        if len == 0 || len > MAX_QUESTION_CHARS {
            return Err(format!(
                "Question must be between 1 and {MAX_QUESTION_CHARS} characters."
            ));
        }

        let body = json!({
            "model": model,
            "messages": [{ "role": "user", "content": question }],
            "web_search_options": { "search_context_size": "low" },
        });

        let response = self
            .http
            .post(API_URL)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    "Request timed out. Perplexity took too long to respond.".to_string()
                } else {
                    format!("Request failed: {e}")
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                401 => "Authentication failed. Check PERPLEXITY_API_KEY.".to_string(),
                429 => "Rate limit exceeded. Wait before trying again.".to_string(),
                _ => format!(
                    "Perplexity returned {status}: {}",
                    response.text().await.unwrap_or_default()
                ),
            });
        }

        let chat: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Unexpected response from Perplexity: {e}"))?;

        chat.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| "Perplexity returned no answer.".to_string())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Perplexity {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(
            Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("PERPLEXITY_API_KEY")
        .map_err(|_| "PERPLEXITY_API_KEY environment variable is not set")?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    if env::args().nth(1).as_deref() == Some("--http") {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("PORT").unwrap_or_else(|_| "8000".into());
        let service: StreamableHttpService<Perplexity, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(Perplexity::new(http.clone(), api_key.clone())),
                Default::default(),
                StreamableHttpServerConfig::default(),
            );
        let router = axum::Router::new().nest_service("/mcp", service);
        let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
        eprintln!("Perplexity MCP listening on http://{host}:{port}/mcp");
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await?;
    } else {
        Perplexity::new(http, api_key)
            .serve(stdio())
            .await?
            .waiting()
            .await?;
    }

    Ok(())
}
