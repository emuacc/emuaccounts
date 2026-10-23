//! OpenAI-compatible chat-completions client with a tool-calling loop.
//!
//! All AI features go through this client. It is configured by API URL +
//! API key + model stored in `RootConfig` (Global Settings → config.json);
//! any OpenAI-compatible server works (LM Studio locally, OpenAI/OpenRouter
//! remotely). Calls may carry tool definitions ([`ToolDef`]); [`AiClient::run`]
//! executes requested tool calls and feeds results back until the model
//! returns a final answer. Models that never call tools work unchanged —
//! the first plain-content reply is returned as-is.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Ceiling on tool-call rounds before giving up on a looping model.
pub const MAX_TOOL_ROUNDS: usize = 16;
/// Tool results are truncated to this many bytes so a chatty tool cannot
/// blow the context window of a small local model.
const TOOL_RESULT_MAX_BYTES: usize = 32 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Local models can take minutes on a long prompt; callers are all
/// fire-and-forget background threads, so a generous read timeout is safe.
/// 300s proved too tight in practice: a document-import prefill (~10K tokens)
/// plus a 21-row JSON extraction on a 27B MLX model exceeded it three times
/// on 2026-08-17.
const READ_TIMEOUT: Duration = Duration::from_secs(900);
const TEMPERATURE: f32 = 0.2;

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("AI is not configured. Set the API URL and model in Global Settings (gear icon).")]
    NotConfigured,
    #[error("Could not reach the AI server at {url}: {detail}")]
    Connect { url: String, detail: String },
    #[error("AI server returned HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("Unexpected AI response: {0}")]
    BadResponse(String),
    #[error("Model exceeded {0} tool-call rounds without a final answer")]
    ToolLoopExceeded(usize),
}

#[derive(Debug, Clone)]
pub struct AiClientConfig {
    /// Base URL including `/v1`, e.g. `http://localhost:1234/v1`.
    pub api_url: String,
    /// Bearer token; empty = no Authorization header (local servers).
    pub api_key: String,
    /// Model id sent with every request.
    pub model: String,
    /// "low"/"medium"/"high" sent as `reasoning_effort`; empty = omitted.
    pub reasoning_effort: String,
}

impl AiClientConfig {
    pub fn from_root(cfg: &crate::root_config::RootConfig) -> Result<Self, AiError> {
        let api_url = cfg.ai_api_url.trim().trim_end_matches('/').to_string();
        let model = cfg.ai_model.trim().to_string();
        if api_url.is_empty() || model.is_empty() {
            return Err(AiError::NotConfigured);
        }
        Ok(Self {
            api_url,
            api_key: cfg.ai_api_key.trim().to_string(),
            model,
            reasoning_effort: cfg.ai_reasoning_effort.trim().to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    /// Arrives as a JSON *string* on the wire, not an object.
    pub arguments: String,
}

/// A function the model may call. `handler` receives the parsed arguments
/// object and returns the tool result text (or an error message, which is
/// fed back to the model rather than aborting the run).
pub struct ToolDef {
    pub name: String,
    pub description: String,
    /// JSON Schema for the arguments object.
    pub parameters: serde_json::Value,
    #[allow(clippy::type_complexity)]
    pub handler: Box<dyn Fn(&serde_json::Value) -> Result<String, String> + Send + Sync>,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    temperature: f32,
    /// Omitted when unset so servers/models without adjustable reasoning
    /// never see the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
    /// Omitted entirely when no tools — some servers reject an empty array.
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolSpecWire<'a>>>,
}

#[derive(Serialize)]
struct ToolSpecWire<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    function: FunctionSpec<'a>,
}

#[derive(Serialize)]
struct FunctionSpec<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

pub struct AiClient {
    cfg: AiClientConfig,
    agent: ureq::Agent,
}

impl AiClient {
    pub fn new(cfg: AiClientConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(CONNECT_TIMEOUT)
            .timeout_read(READ_TIMEOUT)
            .build();
        Self { cfg, agent }
    }

    pub fn from_root_config(cfg: &crate::root_config::RootConfig) -> Result<Self, AiError> {
        Ok(Self::new(AiClientConfig::from_root(cfg)?))
    }

    /// For non-Tauri callers (pipeline fallback, CLI bins): load the AI
    /// settings from config.json in the platform app-data dir.
    pub fn from_disk() -> Result<Self, AiError> {
        let app_dir = crate::root_config::platform_app_data_dir();
        let cfg = crate::root_config::load_root_config(&app_dir);
        Self::from_root_config(&cfg)
    }

    pub fn model(&self) -> &str {
        &self.cfg.model
    }

    /// Run the chat with a tool loop: while the model requests tool calls,
    /// execute each handler, append the assistant message plus one
    /// `role:"tool"` result per call, and re-request. Returns the first
    /// assistant reply that carries no tool calls. Pass `tools: &[]` for a
    /// single-shot request.
    pub fn run(
        &self,
        mut messages: Vec<ChatMessage>,
        tools: &[ToolDef],
        on_step: &(dyn Fn(&str) + Send + Sync),
    ) -> Result<String, AiError> {
        for _ in 0..MAX_TOOL_ROUNDS {
            let msg = self.post_chat(&messages, tools)?;
            let tool_calls = msg.tool_calls.clone().unwrap_or_default();
            if tool_calls.is_empty() {
                return match msg.content {
                    Some(c) if !c.trim().is_empty() => Ok(c),
                    _ => Err(AiError::BadResponse(
                        "assistant reply had no content and no tool calls".to_string(),
                    )),
                };
            }
            // A reply carrying both content and tool calls: the tool calls win.
            messages.push(msg);
            for call in &tool_calls {
                on_step(&format!(
                    "→ tool {} {}",
                    call.function.name,
                    truncate_chars(&call.function.arguments, 200)
                ));
                let result = run_tool(tools, call);
                on_step(&format!("← {} bytes", result.len()));
                messages.push(ChatMessage::tool(
                    call.id.clone(),
                    truncate_bytes(result, TOOL_RESULT_MAX_BYTES),
                ));
            }
        }
        Err(AiError::ToolLoopExceeded(MAX_TOOL_ROUNDS))
    }

    fn post_chat(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDef],
    ) -> Result<ChatMessage, AiError> {
        let url = format!("{}/chat/completions", self.cfg.api_url);
        let tools_wire = if tools.is_empty() {
            None
        } else {
            Some(
                tools
                    .iter()
                    .map(|t| ToolSpecWire {
                        kind: "function",
                        function: FunctionSpec {
                            name: &t.name,
                            description: &t.description,
                            parameters: &t.parameters,
                        },
                    })
                    .collect(),
            )
        };
        let request = ChatRequest {
            model: &self.cfg.model,
            messages,
            temperature: TEMPERATURE,
            reasoning_effort: if self.cfg.reasoning_effort.is_empty() {
                None
            } else {
                Some(&self.cfg.reasoning_effort)
            },
            tools: tools_wire,
        };
        let body = serde_json::to_string(&request)
            .map_err(|e| AiError::BadResponse(format!("failed to serialize request: {e}")))?;

        let mut http = self.agent.post(&url).set("Content-Type", "application/json");
        if !self.cfg.api_key.is_empty() {
            http = http.set("Authorization", &format!("Bearer {}", self.cfg.api_key));
        }
        let response = match http.send_string(&body) {
            Ok(r) => r,
            Err(ureq::Error::Status(status, resp)) => {
                let body = truncate_chars(&resp.into_string().unwrap_or_default(), 500);
                return Err(AiError::Http { status, body });
            }
            Err(ureq::Error::Transport(t)) => {
                return Err(AiError::Connect {
                    url,
                    detail: t.to_string(),
                });
            }
        };
        let text = response
            .into_string()
            .map_err(|e| AiError::BadResponse(format!("failed to read response body: {e}")))?;
        let parsed: ChatResponse = serde_json::from_str(&text).map_err(|e| {
            AiError::BadResponse(format!("{e}; body: {}", truncate_chars(&text, 200)))
        })?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| AiError::BadResponse("response had no choices".to_string()))
    }
}

/// Execute one tool call. Every failure mode becomes a JSON error payload fed
/// back to the model — local models emit malformed calls routinely and can
/// recover when told what went wrong.
fn run_tool(tools: &[ToolDef], call: &ToolCall) -> String {
    let Some(tool) = tools.iter().find(|t| t.name == call.function.name) else {
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        return serde_json::json!({
            "error": format!("unknown tool \"{}\"; available: {}", call.function.name, names.join(", "))
        })
        .to_string();
    };
    let raw = call.function.arguments.trim();
    let raw = if raw.is_empty() { "{}" } else { raw };
    let args: serde_json::Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "error": format!("could not parse tool arguments as JSON: {e}")
            })
            .to_string();
        }
    };
    match (tool.handler)(&args) {
        Ok(s) => s,
        Err(e) => serde_json::json!({ "error": e }).to_string(),
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let head: String = s.chars().take(max).collect();
        format!("{head}…")
    }
}

fn truncate_bytes(s: String, max: usize) -> String {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…[truncated]", &s[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct Stub {
        url: String,
        /// (body, authorization header) per request received.
        requests: Arc<Mutex<Vec<(String, Option<String>)>>>,
    }

    /// Serve the canned (status, body) responses in order, then exit.
    fn spawn_stub(responses: Vec<(u16, String)>) -> Stub {
        let server = tiny_http::Server::http("127.0.0.1:0").expect("bind stub server");
        let port = server.server_addr().to_ip().expect("stub ip").port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        std::thread::spawn(move || {
            for (status, body) in responses {
                let Ok(mut request) = server.recv() else { return };
                let mut buf = String::new();
                let _ = request.as_reader().read_to_string(&mut buf);
                let auth = request
                    .headers()
                    .iter()
                    .find(|h| h.field.equiv("Authorization"))
                    .map(|h| h.value.to_string());
                recorded.lock().unwrap().push((buf, auth));
                let _ = request.respond(
                    tiny_http::Response::from_string(body).with_status_code(status),
                );
            }
        });
        Stub {
            url: format!("http://127.0.0.1:{port}/v1"),
            requests,
        }
    }

    fn client_for(url: &str, key: &str) -> AiClient {
        AiClient::new(AiClientConfig {
            api_url: url.to_string(),
            api_key: key.to_string(),
            model: "test-model".to_string(),
            reasoning_effort: String::new(),
        })
    }

    fn content_reply(content: &str) -> String {
        serde_json::json!({
            "choices": [{"message": {"role": "assistant", "content": content}}]
        })
        .to_string()
    }

    fn tool_call_reply(id: &str, name: &str, arguments: &str) -> String {
        serde_json::json!({
            "choices": [{"message": {"role": "assistant", "content": null,
                "tool_calls": [{"id": id, "type": "function",
                    "function": {"name": name, "arguments": arguments}}]}}]
        })
        .to_string()
    }

    fn echo_upper_tool() -> ToolDef {
        ToolDef {
            name: "echo_upper".to_string(),
            description: "Uppercase the given text".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {"text": {"type": "string"}},
                "required": ["text"]
            }),
            handler: Box::new(|args| {
                args["text"]
                    .as_str()
                    .map(|s| s.to_uppercase())
                    .ok_or_else(|| "missing text argument".to_string())
            }),
        }
    }

    #[test]
    fn happy_path_no_tools() {
        let stub = spawn_stub(vec![(200, content_reply("hello"))]);
        let client = client_for(&stub.url, "");
        let answer = client
            .run(vec![ChatMessage::user("hi")], &[], &|_| {})
            .expect("run");
        assert_eq!(answer, "hello");
        let requests = stub.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        let (body, auth) = &requests[0];
        assert!(auth.is_none(), "no Authorization header expected");
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
        assert!(parsed.get("tools").is_none(), "tools key must be omitted");
        assert!(
            parsed.get("reasoning_effort").is_none(),
            "reasoning_effort must be omitted when unset"
        );
        assert_eq!(parsed["model"], "test-model");
        assert_eq!(parsed["messages"][0]["content"], "hi");
    }

    #[test]
    fn reasoning_effort_sent_when_set() {
        let stub = spawn_stub(vec![(200, content_reply("ok"))]);
        let client = AiClient::new(AiClientConfig {
            api_url: stub.url.clone(),
            api_key: String::new(),
            model: "test-model".to_string(),
            reasoning_effort: "medium".to_string(),
        });
        client
            .run(vec![ChatMessage::user("hi")], &[], &|_| {})
            .expect("run");
        let requests = stub.requests.lock().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&requests[0].0).unwrap();
        assert_eq!(parsed["reasoning_effort"], "medium");
    }

    #[test]
    fn auth_header_sent_when_key_set() {
        let stub = spawn_stub(vec![(200, content_reply("ok"))]);
        let client = client_for(&stub.url, "sk-test");
        client
            .run(vec![ChatMessage::user("hi")], &[], &|_| {})
            .expect("run");
        let requests = stub.requests.lock().unwrap();
        assert_eq!(requests[0].1.as_deref(), Some("Bearer sk-test"));
    }

    #[test]
    fn tool_round_trip() {
        let stub = spawn_stub(vec![
            (200, tool_call_reply("call_1", "echo_upper", r#"{"text":"abc"}"#)),
            (200, content_reply("done")),
        ]);
        let client = client_for(&stub.url, "");
        let steps: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let steps_sink = steps.clone();
        let answer = client
            .run(
                vec![ChatMessage::user("hi")],
                &[echo_upper_tool()],
                &move |s| steps_sink.lock().unwrap().push(s.to_string()),
            )
            .expect("run");
        assert_eq!(answer, "done");

        let requests = stub.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        let first: serde_json::Value = serde_json::from_str(&requests[0].0).unwrap();
        assert_eq!(first["tools"][0]["function"]["name"], "echo_upper");
        let second: serde_json::Value = serde_json::from_str(&requests[1].0).unwrap();
        let messages = second["messages"].as_array().unwrap();
        // user, assistant (tool_calls), tool result
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["tool_calls"][0]["id"], "call_1");
        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "call_1");
        assert_eq!(messages[2]["content"], "ABC");

        let steps = steps.lock().unwrap();
        assert!(steps.iter().any(|s| s.contains("echo_upper")));
    }

    #[test]
    fn http_error_status() {
        let stub = spawn_stub(vec![(500, "boom".to_string())]);
        let client = client_for(&stub.url, "");
        let err = client
            .run(vec![ChatMessage::user("hi")], &[], &|_| {})
            .unwrap_err();
        match err {
            AiError::Http { status, body } => {
                assert_eq!(status, 500);
                assert_eq!(body, "boom");
            }
            other => panic!("expected Http error, got {other:?}"),
        }
    }

    #[test]
    fn connect_error() {
        // Bind then immediately drop a listener so the port is closed.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let client = client_for(&format!("http://127.0.0.1:{port}/v1"), "");
        let err = client
            .run(vec![ChatMessage::user("hi")], &[], &|_| {})
            .unwrap_err();
        assert!(matches!(err, AiError::Connect { .. }), "got {err:?}");
    }

    #[test]
    fn bad_response_body() {
        let stub = spawn_stub(vec![(200, "not json".to_string())]);
        let client = client_for(&stub.url, "");
        let err = client
            .run(vec![ChatMessage::user("hi")], &[], &|_| {})
            .unwrap_err();
        assert!(matches!(err, AiError::BadResponse(_)), "got {err:?}");
    }

    #[test]
    fn empty_choices_is_bad_response() {
        let stub = spawn_stub(vec![(200, r#"{"choices":[]}"#.to_string())]);
        let client = client_for(&stub.url, "");
        let err = client
            .run(vec![ChatMessage::user("hi")], &[], &|_| {})
            .unwrap_err();
        assert!(matches!(err, AiError::BadResponse(_)), "got {err:?}");
    }

    #[test]
    fn tool_loop_cap() {
        let reply = tool_call_reply("call_n", "echo_upper", r#"{"text":"x"}"#);
        let stub = spawn_stub(vec![(200, reply); MAX_TOOL_ROUNDS]);
        let client = client_for(&stub.url, "");
        let err = client
            .run(vec![ChatMessage::user("hi")], &[echo_upper_tool()], &|_| {})
            .unwrap_err();
        assert!(matches!(err, AiError::ToolLoopExceeded(_)), "got {err:?}");
        assert_eq!(stub.requests.lock().unwrap().len(), MAX_TOOL_ROUNDS);
    }

    #[test]
    fn bad_tool_arguments_fed_back() {
        let stub = spawn_stub(vec![
            (200, tool_call_reply("call_1", "echo_upper", "not json")),
            (200, content_reply("recovered")),
        ]);
        let client = client_for(&stub.url, "");
        let answer = client
            .run(vec![ChatMessage::user("hi")], &[echo_upper_tool()], &|_| {})
            .expect("run");
        assert_eq!(answer, "recovered");
        let requests = stub.requests.lock().unwrap();
        let second: serde_json::Value = serde_json::from_str(&requests[1].0).unwrap();
        let tool_msg = &second["messages"][2];
        assert_eq!(tool_msg["role"], "tool");
        assert!(tool_msg["content"]
            .as_str()
            .unwrap()
            .contains("could not parse tool arguments"));
    }

    #[test]
    fn unknown_tool_fed_back() {
        let stub = spawn_stub(vec![
            (200, tool_call_reply("call_1", "nope", "{}")),
            (200, content_reply("recovered")),
        ]);
        let client = client_for(&stub.url, "");
        let answer = client
            .run(vec![ChatMessage::user("hi")], &[echo_upper_tool()], &|_| {})
            .expect("run");
        assert_eq!(answer, "recovered");
        let requests = stub.requests.lock().unwrap();
        let second: serde_json::Value = serde_json::from_str(&requests[1].0).unwrap();
        let content = second["messages"][2]["content"].as_str().unwrap();
        assert!(content.contains("unknown tool"), "got: {content}");
        assert!(content.contains("echo_upper"), "got: {content}");
    }

    #[test]
    fn empty_arguments_treated_as_empty_object() {
        let stub = spawn_stub(vec![
            (200, tool_call_reply("call_1", "no_args", "")),
            (200, content_reply("fine")),
        ]);
        let tool = ToolDef {
            name: "no_args".to_string(),
            description: "no arguments".to_string(),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
            handler: Box::new(|_| Ok("ran".to_string())),
        };
        let client = client_for(&stub.url, "");
        let answer = client
            .run(vec![ChatMessage::user("hi")], &[tool], &|_| {})
            .expect("run");
        assert_eq!(answer, "fine");
        let requests = stub.requests.lock().unwrap();
        let second: serde_json::Value = serde_json::from_str(&requests[1].0).unwrap();
        assert_eq!(second["messages"][2]["content"], "ran");
    }

    #[test]
    fn not_configured_from_empty_root_config() {
        let root = crate::root_config::RootConfig::default();
        assert!(matches!(
            AiClientConfig::from_root(&root),
            Err(AiError::NotConfigured)
        ));

        let mut url_only = crate::root_config::RootConfig::default();
        url_only.ai_api_url = "http://localhost:1234/v1".to_string();
        assert!(matches!(
            AiClientConfig::from_root(&url_only),
            Err(AiError::NotConfigured)
        ));

        let mut full = crate::root_config::RootConfig::default();
        full.ai_api_url = "http://localhost:1234/v1/".to_string();
        full.ai_model = "m".to_string();
        let cfg = AiClientConfig::from_root(&full).expect("configured");
        assert_eq!(cfg.api_url, "http://localhost:1234/v1");
    }

    #[test]
    fn not_configured_error_mentions_global_settings() {
        assert!(AiError::NotConfigured.to_string().contains("Global Settings"));
    }
}
