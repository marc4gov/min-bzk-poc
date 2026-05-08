pub mod mesh;
pub mod registry;

use autoagents::async_trait;
use autoagents::core::agent::memory::SlidingWindowMemory;
use autoagents::core::agent::prebuilt::executor::{ReActAgent, ReActAgentOutput};
use autoagents::core::agent::task::Task;
use autoagents::core::agent::{AgentBuilder, AgentDeriveT, AgentHooks, DirectAgent};
use autoagents::core::tool::{ToolCallError, ToolRuntime};
use autoagents::llm::backends::ollama::Ollama;
use autoagents::llm::builder::LLMBuilder;
use autoagents::prelude::*;
use autoagents_derive::{agent, tool, AgentHooks, AgentOutput, ToolInput};
use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use futures::StreamExt;
use registry::{default_ollama_model, is_registered_id, normalize_route, ollama_base_url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

// ============================================================================
// TOOLS
// ============================================================================

#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct ReadFileArgs {
    #[input(description = "Pad naar het bestand")]
    path: String,
}

#[tool(
    name = "read_file",
    description = "Lees de inhoud van een bestand. Gebruik dit wanneer je bestandsinhoud nodig hebt.",
    input = ReadFileArgs,
)]
pub struct ReadFileTool;

#[async_trait]
impl ToolRuntime for ReadFileTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: ReadFileArgs =
            serde_json::from_value(args).map_err(|e| ToolCallError::RuntimeError(e.into()))?;
        let path = args.path.strip_prefix("/").unwrap_or(&args.path);

        tokio::fs::read_to_string(path)
            .await
            .map(|content| serde_json::json!({"content": content}))
            .map_err(|e| ToolCallError::RuntimeError(e.into()))
    }
}

#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct ListDirArgs {
    #[input(description = "Pad naar de directory (leeg voor huidige)")]
    path: String,
}

#[tool(
    name = "list_dir",
    description = "Lijst bestanden in een directory. Gebruik dit om bestanden en mappen te bekijken.",
    input = ListDirArgs,
)]
pub struct ListDirTool;

#[async_trait]
impl ToolRuntime for ListDirTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: ListDirArgs =
            serde_json::from_value(args).map_err(|e| ToolCallError::RuntimeError(e.into()))?;

        let path = if args.path.is_empty() {
            "."
        } else {
            &args.path
        };
        let path = path.strip_prefix("/").unwrap_or(path);

        let mut entries = tokio::fs::read_dir(path)
            .await
            .map_err(|e| ToolCallError::RuntimeError(e.into()))?;

        let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);

            files.push(serde_json::json!({
                "name": name,
                "type": if is_dir { "directory" } else { "file" }
            }));
        }

        Ok(serde_json::json!({ "entries": files }))
    }
}

#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct ShellArgs {
    #[input(description = "Het uit te voeren commando")]
    command: String,
}

#[tool(
    name = "shell",
    description = "Voer een shell commando uit. Alleen read-only commando's (ls, pwd, cat, grep, find, date, etc).",
    input = ShellArgs,
)]
pub struct ShellTool;

#[async_trait]
impl ToolRuntime for ShellTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: ShellArgs =
            serde_json::from_value(args).map_err(|e| ToolCallError::RuntimeError(e.into()))?;

        // Safety check
        let dangerous = [
            "rm ", "mv ", "cp ", "dd ", "mkfs", "format", "del ", "shutdown", "reboot", "su ",
            "sudo",
        ];
        let cmd_lower = args.command.to_lowercase();
        for dangerous_cmd in dangerous {
            if cmd_lower.contains(dangerous_cmd) {
                return Err(ToolCallError::RuntimeError(
                    format!("Commando geblokkeerd: {}", dangerous_cmd.trim()).into(),
                ));
            }
        }

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .output()
            .await
            .map_err(|e| ToolCallError::RuntimeError(e.into()))?;

        Ok(serde_json::json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}

#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct WebSearchArgs {
    #[input(description = "De zoekterm")]
    query: String,
}

#[tool(
    name = "web_search",
    description = "Zoek op het web voor actuele informatie. Gebruik dit voor vragen over nieuws, weer, actuele data.",
    input = WebSearchArgs,
)]
pub struct WebSearchTool;

#[async_trait]
impl ToolRuntime for WebSearchTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: WebSearchArgs =
            serde_json::from_value(args).map_err(|e| ToolCallError::RuntimeError(e.into()))?;

        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(&args.query)
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ToolCallError::RuntimeError(e.into()))?;

        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| ToolCallError::RuntimeError(e.into()))?;

        if !resp.status().is_success() {
            return Ok(serde_json::json!({"results": []}));
        }

        let html = resp
            .text()
            .await
            .map_err(|e| ToolCallError::RuntimeError(e.into()))?;

        let results = extract_search_results(&html);
        Ok(serde_json::json!({ "results": results }))
    }
}

fn extract_search_results(html: &str) -> Vec<Value> {
    use regex::Regex;

    let link_re =
        Regex::new(r#"<a[^>]*class="result__a"[^>]*href="([^"]+)"[^>]*>([^<]+)</a>"#).unwrap();

    let mut results = Vec::new();
    for cap in link_re.captures_iter(html).take(5) {
        if let (Some(url), Some(title)) = (cap.get(1), cap.get(2)) {
            results.push(serde_json::json!({
                "title": strip_html(title.as_str()),
                "url": url.as_str()
            }));
        }
    }

    if results.is_empty() {
        let any_link = Regex::new(r#"<a[^>]*href="(https?:[^"]+)"[^>]*>([^<]+)</a>"#).unwrap();
        for cap in any_link.captures_iter(html).take(5) {
            if let (Some(url), Some(title)) = (cap.get(1), cap.get(2)) {
                let title = strip_html(title.as_str());
                if !title.is_empty() && title.len() > 3 {
                    results.push(serde_json::json!({ "title": title, "url": url.as_str() }));
                }
            }
        }
    }

    results
}

fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .trim()
        .to_string()
}

// ============================================================================
// AGENT OUTPUTS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, AgentOutput)]
pub struct SimpleOutput {
    #[output(description = "Het antwoord")]
    pub answer: String,
}

impl From<ReActAgentOutput> for SimpleOutput {
    fn from(output: ReActAgentOutput) -> Self {
        // Try to parse as JSON first, in case the LLM returned JSON
        let content = if let Ok(value) = serde_json::from_str::<Value>(&output.response) {
            // If it's a JSON object with an "answer" field, extract it
            if let Some(answer) = value.get("answer").and_then(|v| v.as_str()) {
                answer.to_string()
            } else {
                // Otherwise use the full response as string
                output.response.clone()
            }
        } else {
            output.response.clone()
        };
        SimpleOutput { answer: content }
    }
}

// ============================================================================
// ROUTING OUTPUT
// ============================================================================

#[derive(Debug, Serialize, Deserialize, AgentOutput)]
pub struct RoutingOutput {
    #[output(
        description = "JSON: {\"agent\":\"<id>\"} — id ∈ code|schrijf|tools|research|translate|analyze|docs|brainstorm|data|general"
    )]
    pub agent: String,
}

impl From<ReActAgentOutput> for RoutingOutput {
    fn from(output: ReActAgentOutput) -> Self {
        let raw = if let Ok(value) = serde_json::from_str::<Value>(&output.response) {
            value
                .get("agent")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| output.response.clone())
        } else {
            output.response.clone()
        };
        RoutingOutput {
            agent: normalize_route(&raw).to_string(),
        }
    }
}

// ============================================================================
// AGENTS
// ============================================================================

// Orchestrator — output is strikt JSON; details staan ook in registry::SPECIALISTS
#[agent(
    name = "orchestrator",
    description = "Je routeert naar precies ÉÉN specialist. Geen uitleg, geen markdown.

Antwoord UITSLUITEND met dit JSON-object op één regel:
{\"agent\":\"<id>\"}

Toegestane id's staan centraal; gebruik alleen deze exacte kleine letters:
code, schrijf, tools, research, translate, analyze, docs, brainstorm, data, general
(kort: code=bouwen/debug; schrijf=NL-teksten; tools=mix filesystem+shell+web; research=live web;
translate=taalkeuze; analyze=inhoud/structuur bestanden; docs=readme/API-md; brainstorm=ideeen;
data=CSV/JSON lezen; general=zonder tools).",
    output = RoutingOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct OrchestratorAgent {}

// Code Agent
#[agent(
    name = "code_expert",
    description = "Je bent een code expert. Je helpt met code schrijven, debugging en technische vragen. Je spreekt Nederlands. Geef duidelijke, pragmatische antwoorden met code voorbeelden waar relevant.",
    tools = [ReadFileTool, ListDirTool, ShellTool],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct CodeAgent {}

// Schrijf Agent
#[agent(
    name = "schrijf_expert",
    description = "Je bent een tekst expert die helpt met het schrijven en redigeren van Nederlandse teksten. Je helpt met e-mails, brieven, rapporten, creatieve teksten en samenvattingen. Je schrijft in helder, correct Nederlands.",
    tools = [],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct SchrijfAgent {}

// Tools Agent
#[agent(
    name = "tools_expert",
    description = "Je bent een tools expert. Je helpt met bestanden lezen, directory's verkennen, web searches en shell commando's. Je spreekt Nederlands. Wees behulpzaam en specifiek in je antwoorden.",
    tools = [ReadFileTool, ListDirTool, ShellTool, WebSearchTool],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct ToolsAgent {}

// Research — bewust klein: alleen web_search
#[agent(
    name = "research_expert",
    description = "Je bent een research-specialist: actuele informatie via web_search. Nederlands. Verwijs naar bronnen waar zinvol; verzin geen feiten als de zoekresultaten tekortschieten.",
    tools = [WebSearchTool],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct ResearchAgent {}

#[agent(
    name = "translate_expert",
    description = "Vertaal- en taalspecialist. Pas toon aan (informeel/formeel). Geen hallucinaties over cultuur-gevoelige feiten; bij twijfel zeg het.",
    tools = [],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct TranslateAgent {}

#[agent(
    name = "analyze_expert",
    description = "Analyse-specialist voor bestanden en mappen op schijf: inhoud bekijken, structuur beschrijven, samenvatten. Nederlands.",
    tools = [ReadFileTool, ListDirTool],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct AnalyzeAgent {}

#[agent(
    name = "docs_expert",
    description = "Documentatie-specialist voor ontwikkelaars: README, Markdown, endpoint-beschrijvingen. Nederlands waar passend.",
    tools = [ReadFileTool, ShellTool],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct DocsAgent {}

#[agent(
    name = "brainstorm_expert",
    description = "Creatieve brainstorm: ideeen, invalshoeken, frameworks. Nederlands. Geen valse bronnen.",
    tools = [],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct BrainstormAgent {}

#[agent(
    name = "data_expert",
    description = "Helpt bij het begrijpen van gestructureerde data in bestanden (CSV, JSON, logs). Je leest bestanden via read_file; extrapoleer voorzichtig bij grote bestanden.",
    tools = [ReadFileTool],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct DataAgent {}

// General Agent
#[agent(
    name = "general_assistant",
    description = "Je bent een behulpzage AI assistant. Je spreekt Nederlands. Geef korte, directe antwoorden op algemene vragen.",
    tools = [],
    output = SimpleOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct GeneralAgent {}

// ============================================================================
// HTTP API
// ============================================================================

#[derive(Clone)]
pub struct AppState {
    pub event_tx: broadcast::Sender<AgentEvent>,
}

#[derive(Debug, Clone)]
pub struct AgentRequest {
    pub message: String,
    pub selected_agent: Option<String>,
    pub response_tx: mpsc::UnboundedSender<AgentResponse>,
}

#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub agents_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    Connected,
    Started {
        agent: String,
        timestamp: i64,
    },
    OrchestratorDecision {
        target_agent: String,
        reasoning: String,
        timestamp: i64,
    },
    Thinking {
        agent: String,
        timestamp: i64,
    },
    ToolUse {
        agent: String,
        tool: String,
        args: serde_json::Value,
        timestamp: i64,
    },
    ToolResult {
        agent: String,
        tool: String,
        result: serde_json::Value,
        timestamp: i64,
    },
    Response {
        agent: String,
        content: String,
        timestamp: i64,
    },
    Streaming {
        agent: String,
        delta: String,
        content: String,
        done: bool,
    },
    Error {
        error: String,
        agent: Option<String>,
        timestamp: i64,
    },
    /// Actor-mesh (Entry, experts, delegatie, Ollama, heuristiek).
    Mesh {
        actor: String,
        phase: String,
        detail: String,
        #[serde(skip)]
        model: Option<String>,
        timestamp: i64,
    },
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub selected_agent: Option<String>,
    /// Ollama model tag (bv. `qwen3-coder:latest`). Leeftijd: zoals gekozen in de frontend per bericht.
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub content: String,
    pub agents_used: Vec<String>,
    pub tokens_used: u32,
    /// Welk Ollama-model effectief gebruikt is voor deze chatronde.
    pub model_used: String,
}

async fn run_agent_streaming<A>(
    agent_def: A,
    llm: &Arc<Ollama>,
    message: String,
    memory_size: usize,
    agent_label: &str,
    event_tx: &broadcast::Sender<AgentEvent>,
) -> Result<SimpleOutput, anyhow::Error>
where
    A: AgentDeriveT<Output = SimpleOutput> + AgentHooks + Default + Clone + Send + Sync + 'static,
    SimpleOutput: From<ReActAgentOutput>,
{
    let react_agent = ReActAgent::new(agent_def);
    let agent_handle = AgentBuilder::<_, DirectAgent>::new(react_agent)
        .llm(llm.clone())
        .memory(Box::new(SlidingWindowMemory::new(memory_size)))
        .stream(true)
        .build()
        .await?;

    let mut stream = agent_handle
        .agent
        .run_stream(Task::new(message))
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut accumulated = String::new();
    while let Some(item) = stream.next().await {
        let out = match item {
            Ok(o) => o,
            Err(e) => return Err(anyhow::anyhow!("{}", e)),
        };
        let text = &out.answer;
        if accumulated.is_empty() {
            accumulated = text.clone();
        } else if !text.is_empty() && text.starts_with(&accumulated) {
            accumulated = text.clone();
        } else if !text.is_empty() {
            accumulated.push_str(text);
        }

        let _ = event_tx.send(AgentEvent::Streaming {
            agent: agent_label.to_string(),
            delta: text.clone(),
            content: accumulated.clone(),
            done: false,
        });
    }

    let _ = event_tx.send(AgentEvent::Streaming {
        agent: agent_label.to_string(),
        delta: String::new(),
        content: accumulated.clone(),
        done: true,
    });

    Ok(SimpleOutput {
        answer: accumulated,
    })
}

async fn run_agent<A>(
    agent_def: A,
    llm: &Arc<Ollama>,
    message: String,
    memory_size: usize,
) -> Result<SimpleOutput, anyhow::Error>
where
    A: AgentDeriveT<Output = SimpleOutput> + AgentHooks + Default + Clone + Send + Sync + 'static,
    SimpleOutput: From<ReActAgentOutput>,
{
    let react_agent = ReActAgent::new(agent_def);
    let agent_handle = AgentBuilder::<_, DirectAgent>::new(react_agent)
        .llm(llm.clone())
        .memory(Box::new(SlidingWindowMemory::new(memory_size)))
        .build()
        .await?;

    Ok(agent_handle.agent.run(Task::new(message)).await?)
}

fn resolve_chat_model(req: &ChatRequest) -> String {
    if let Some(ref m) = req.model {
        let t = m.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    default_ollama_model()
}

fn build_ollama(model: &str) -> anyhow::Result<Arc<Ollama>> {
    LLMBuilder::<Ollama>::new()
        .base_url(ollama_base_url())
        .model(model)
        .build()
        .map_err(|e| anyhow::anyhow!("{}", e))
}

// Helper to run orchestrator
async fn run_orchestrator(
    llm: &Arc<Ollama>,
    message: String,
) -> Result<RoutingOutput, anyhow::Error> {
    let react_agent = ReActAgent::new(OrchestratorAgent::default());
    let agent_handle = AgentBuilder::<_, DirectAgent>::new(react_agent)
        .llm(llm.clone())
        .memory(Box::new(SlidingWindowMemory::new(5)))
        .build()
        .await?;

    Ok(agent_handle.agent.run(Task::new(message)).await?)
}

pub async fn create_app() -> anyhow::Result<Router> {
    let default_llm = build_ollama(&default_ollama_model())?;

    // Create channels
    let (_request_tx, mut request_rx) = mpsc::unbounded_channel::<AgentRequest>();
    let (event_tx, _) = broadcast::channel::<AgentEvent>(100);

    // Spawn request handler
    let llm_clone = default_llm.clone();
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        while let Some(req) = request_rx.recv().await {
            let _ = event_tx_clone.send(AgentEvent::Started {
                agent: "system".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            });

            let result = process_request(&req, &llm_clone).await;

            match result {
                Ok(response) => {
                    let _ = event_tx_clone.send(AgentEvent::Response {
                        agent: response.agents_used.join(","),
                        content: response.content.clone(),
                        timestamp: chrono::Utc::now().timestamp(),
                    });
                    let _ = req.response_tx.send(response);
                }
                Err(e) => {
                    let error_msg = format!("Agent fout: {}", e);
                    let _ = event_tx_clone.send(AgentEvent::Error {
                        error: error_msg.clone(),
                        agent: None,
                        timestamp: chrono::Utc::now().timestamp(),
                    });
                    let _ = req.response_tx.send(AgentResponse {
                        content: error_msg,
                        agents_used: vec![],
                    });
                }
            }
        }
    });

    let state = AppState { event_tx };
    Ok(Router::new()
        .route("/health", get(health_check))
        .route("/api/agents", get(list_agents))
        .route("/api/mesh/demo", post(mesh_demo_endpoint))
        .route("/api/mesh/demo/document", post(mesh_demo_document_endpoint))
        .route("/api/mesh/improve", post(mesh_improve_document_endpoint))
        .route("/api/ollama/models", get(list_ollama_models_endpoint))
        .route("/api/chat", post(chat))
        .route("/ws", get(websocket_handler))
        .with_state(state))
}

#[derive(Debug, Deserialize)]
struct MeshDemoHttpBody {
    query: String,
    #[serde(default = "default_mesh_demo_http_timeout")]
    timeout_secs: u64,
    /// Alle mesh-experts spawnen en bij Entry registreren (geen kleine Rust+Frontend-demo).
    #[serde(default)]
    full_mesh: bool,
}

#[derive(Debug, Deserialize)]
struct MeshDemoDocumentBody {
    urls: Vec<String>,
    #[serde(default = "default_mesh_demo_http_timeout_document")]
    timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
struct MeshImproveDocumentBody {
    content: String,
    filename: String,
    instructions: String,
    #[serde(default = "default_mesh_improve_http_timeout")]
    timeout_secs: u64,
    #[serde(default)]
    model: Option<String>,
}

fn default_mesh_demo_http_timeout() -> u64 {
    30
}

fn default_mesh_demo_http_timeout_document() -> u64 {
    120
}

fn default_mesh_improve_http_timeout() -> u64 {
    120
}

async fn mesh_demo_endpoint(
    State(state): State<AppState>,
    Json(body): Json<MeshDemoHttpBody>,
) -> Json<crate::mesh::MeshDemoResult> {
    Json(
        crate::mesh::mesh_run_demo_query(
            body.query.trim(),
            body.timeout_secs,
            Some(state.event_tx.clone()),
            body.full_mesh,
        )
        .await,
    )
}

async fn mesh_demo_document_endpoint(
    State(state): State<AppState>,
    Json(body): Json<MeshDemoDocumentBody>,
) -> Json<crate::mesh::MeshDemoResult> {
    Json(
        crate::mesh::mesh_run_document_pipeline(
            body.urls,
            body.timeout_secs,
            Some(state.event_tx.clone()),
        )
        .await,
    )
}

async fn mesh_improve_document_endpoint(
    State(state): State<AppState>,
    Json(body): Json<MeshImproveDocumentBody>,
) -> Json<crate::mesh::MeshDemoResult> {
    // Set the global model override for this request
    crate::mesh::set_mesh_model(body.model.clone());

    let result = crate::mesh::mesh_improve_uploaded_document(
        body.content,
        body.filename,
        body.instructions,
        body.timeout_secs,
        body.model,
        Some(state.event_tx.clone()),
    )
    .await;

    // Clear the model override after the request
    crate::mesh::set_mesh_model(None);

    Json(result)
}

async fn process_request(
    req: &AgentRequest,
    llm: &Arc<Ollama>,
) -> Result<AgentResponse, anyhow::Error> {
    let mut agents_used = Vec::new();

    let agent_choice: &'static str = if let Some(ref selected) = req.selected_agent {
        if is_registered_id(selected) {
            normalize_route(selected)
        } else {
            let routing = run_orchestrator(llm, req.message.clone()).await?;
            agents_used.push("orchestrator".to_string());
            normalize_route(&routing.agent)
        }
    } else {
        let routing = run_orchestrator(llm, req.message.clone()).await?;
        agents_used.push("orchestrator".to_string());
        normalize_route(&routing.agent)
    };

    // Route to appropriate agent
    let output = match agent_choice {
        "code" => {
            agents_used.push("code".to_string());
            run_agent(CodeAgent::default(), llm, req.message.clone(), 20).await?
        }
        "schrijf" => {
            agents_used.push("schrijf".to_string());
            run_agent(SchrijfAgent::default(), llm, req.message.clone(), 20).await?
        }
        "tools" => {
            agents_used.push("tools".to_string());
            run_agent(ToolsAgent::default(), llm, req.message.clone(), 20).await?
        }
        "research" => {
            agents_used.push("research".to_string());
            run_agent(ResearchAgent::default(), llm, req.message.clone(), 20).await?
        }
        "translate" => {
            agents_used.push("translate".to_string());
            run_agent(TranslateAgent::default(), llm, req.message.clone(), 20).await?
        }
        "analyze" => {
            agents_used.push("analyze".to_string());
            run_agent(AnalyzeAgent::default(), llm, req.message.clone(), 20).await?
        }
        "docs" => {
            agents_used.push("docs".to_string());
            run_agent(DocsAgent::default(), llm, req.message.clone(), 20).await?
        }
        "brainstorm" => {
            agents_used.push("brainstorm".to_string());
            run_agent(BrainstormAgent::default(), llm, req.message.clone(), 20).await?
        }
        "data" => {
            agents_used.push("data".to_string());
            run_agent(DataAgent::default(), llm, req.message.clone(), 20).await?
        }
        _ => {
            agents_used.push("general".to_string());
            run_agent(GeneralAgent::default(), llm, req.message.clone(), 20).await?
        }
    };

    Ok(AgentResponse {
        content: output.answer,
        agents_used,
    })
}

async fn process_request_with_events(
    req: &ChatRequest,
    llm: &Arc<Ollama>,
    event_tx: &broadcast::Sender<AgentEvent>,
) -> Result<AgentResponse, anyhow::Error> {
    let mut agents_used = Vec::new();

    let agent_choice: &'static str = if let Some(ref selected) = req.selected_agent {
        if is_registered_id(selected) {
            normalize_route(selected)
        } else {
            let _ = event_tx.send(AgentEvent::Started {
                agent: "orchestrator".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            });
            let routing = run_orchestrator(llm, req.message.clone()).await?;
            agents_used.push("orchestrator".to_string());

            let _ = event_tx.send(AgentEvent::OrchestratorDecision {
                target_agent: routing.agent.clone(),
                reasoning: format!("Route based on query analysis"),
                timestamp: chrono::Utc::now().timestamp(),
            });

            normalize_route(&routing.agent)
        }
    } else {
        let _ = event_tx.send(AgentEvent::Started {
            agent: "orchestrator".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        });
        let routing = run_orchestrator(llm, req.message.clone()).await?;
        agents_used.push("orchestrator".to_string());

        let _ = event_tx.send(AgentEvent::OrchestratorDecision {
            target_agent: routing.agent.clone(),
            reasoning: format!("Route based on query analysis"),
            timestamp: chrono::Utc::now().timestamp(),
        });

        normalize_route(&routing.agent)
    };

    let _ = event_tx.send(AgentEvent::Started {
        agent: agent_choice.to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    });

    let msg = req.message.clone();
    let output = match agent_choice {
        "code" => {
            agents_used.push("code".to_string());
            // Code agent uses tools, disable streaming for Ollama
            let result = run_agent(CodeAgent::default(), llm, msg, 20).await?;
            let _ = event_tx.send(AgentEvent::Streaming {
                agent: "code".to_string(),
                delta: String::new(),
                content: result.answer.clone(),
                done: true,
            });
            result
        }
        "schrijf" => {
            agents_used.push("schrijf".to_string());
            run_agent_streaming(SchrijfAgent::default(), llm, msg, 20, "schrijf", event_tx).await?
        }
        "tools" => {
            agents_used.push("tools".to_string());
            // Tools agent doesn't support streaming with Ollama
            let result = run_agent(ToolsAgent::default(), llm, msg, 20).await?;
            let _ = event_tx.send(AgentEvent::Streaming {
                agent: "tools".to_string(),
                delta: String::new(),
                content: result.answer.clone(),
                done: true,
            });
            result
        }
        "research" => {
            agents_used.push("research".to_string());
            // Research agent uses tools, disable streaming for Ollama
            let result = run_agent(ResearchAgent::default(), llm, msg, 20).await?;
            let _ = event_tx.send(AgentEvent::Streaming {
                agent: "research".to_string(),
                delta: String::new(),
                content: result.answer.clone(),
                done: true,
            });
            result
        }
        "translate" => {
            agents_used.push("translate".to_string());
            run_agent_streaming(
                TranslateAgent::default(),
                llm,
                msg,
                20,
                "translate",
                event_tx,
            )
            .await?
        }
        "analyze" => {
            agents_used.push("analyze".to_string());
            // Analyze agent uses tools, disable streaming for Ollama
            let result = run_agent(AnalyzeAgent::default(), llm, msg, 20).await?;
            let _ = event_tx.send(AgentEvent::Streaming {
                agent: "analyze".to_string(),
                delta: String::new(),
                content: result.answer.clone(),
                done: true,
            });
            result
        }
        "docs" => {
            agents_used.push("docs".to_string());
            run_agent_streaming(DocsAgent::default(), llm, msg, 20, "docs", event_tx).await?
        }
        "brainstorm" => {
            agents_used.push("brainstorm".to_string());
            run_agent_streaming(
                BrainstormAgent::default(),
                llm,
                msg,
                20,
                "brainstorm",
                event_tx,
            )
            .await?
        }
        "data" => {
            agents_used.push("data".to_string());
            run_agent_streaming(DataAgent::default(), llm, msg, 20, "data", event_tx).await?
        }
        _ => {
            agents_used.push("general".to_string());
            run_agent_streaming(GeneralAgent::default(), llm, msg, 20, "general", event_tx).await?
        }
    };

    Ok(AgentResponse {
        content: output.answer,
        agents_used,
    })
}

async fn list_ollama_models_endpoint() -> Json<serde_json::Value> {
    #[derive(Debug, serde::Deserialize)]
    struct OllamaTags {
        models: Vec<OllamaModelRow>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct OllamaModelRow {
        name: String,
    }

    let base_owned = ollama_base_url();
    let base = base_owned.trim_end_matches('/');
    let url = format!("{}/api/tags", base);
    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => match resp.json::<OllamaTags>().await {
            Ok(body) => {
                let models: Vec<_> = body
                    .models
                    .into_iter()
                    .map(|m| serde_json::json!({ "name": m.name }))
                    .collect();
                Json(serde_json::json!({ "ok": true, "models": models }))
            }
            Err(_) => Json(serde_json::json!({
                "ok": false,
                "models": [],
                "error": "parse_error"
            })),
        },
        _ => Json(serde_json::json!({
            "ok": false,
            "models": [],
            "error": "ollama_unreachable"
        })),
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "ollama_available": true,
        "openai_available": false
    }))
}

async fn list_agents() -> Json<Vec<serde_json::Value>> {
    Json(registry::agents_http_json())
}

async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, axum::http::StatusCode> {
    let event_tx = state.event_tx.clone();
    let model_used = resolve_chat_model(&req);

    let llm =
        build_ollama(&model_used).map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Emit start event
    let _ = event_tx.send(AgentEvent::Started {
        agent: "system".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    });

    let result = process_request_with_events(&req, &llm, &event_tx).await;

    match result {
        Ok(response) => {
            // Emit completion event
            let _ = event_tx.send(AgentEvent::Response {
                agent: response.agents_used.join(","),
                content: response.content.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            });
            Ok(Json(ChatResponse {
                content: response.content,
                agents_used: response.agents_used,
                tokens_used: 0,
                model_used,
            }))
        }
        Err(e) => {
            let error_msg = format!("Agent fout: {}", e);
            let _ = event_tx.send(AgentEvent::Error {
                error: error_msg.clone(),
                agent: None,
                timestamp: chrono::Utc::now().timestamp(),
            });
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn websocket_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(mut socket: axum::extract::ws::WebSocket, state: AppState) {
    let mut event_rx = state.event_tx.subscribe();

    // Send welcome
    let welcome = serde_json::to_string(&AgentEvent::Connected).unwrap();
    let _ = socket.send(Message::Text(welcome)).await;

    loop {
        tokio::select! {
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(req) = serde_json::from_str::<ChatRequest>(&text) {
                            let model_used = resolve_chat_model(&req);
                            let llm = match build_ollama(&model_used) {
                                Ok(v) => v,
                                Err(e) => {
                                    let ev = serde_json::to_string(&AgentEvent::Error {
                                        error: format!("Ollama: {}", e),
                                        agent: None,
                                        timestamp: chrono::Utc::now().timestamp(),
                                    }).unwrap();
                                    let _ = socket.send(Message::Text(ev)).await;
                                    continue;
                                }
                            };

                            let result = process_request_with_events(
                                &req,
                                &llm,
                                &state.event_tx,
                            ).await;

                            match result {
                                Ok(response) => {
                                    let event = AgentEvent::Response {
                                        agent: response.agents_used.join(","),
                                        content: response.content,
                                        timestamp: chrono::Utc::now().timestamp(),
                                    };
                                    let _ = socket.send(Message::Text(serde_json::to_string(&event).unwrap())).await;
                                }
                                Err(e) => {
                                    let event = AgentEvent::Error {
                                        error: format!("Agent fout: {}", e),
                                        agent: None,
                                        timestamp: chrono::Utc::now().timestamp(),
                                    };
                                    let _ = socket.send(Message::Text(serde_json::to_string(&event).unwrap())).await;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            // Forward broadcast events to WebSocket
            event_result = event_rx.recv() => {
                if let Ok(event) = event_result {
                    let json = serde_json::to_string(&event).unwrap();
                    let _ = socket.send(Message::Text(json)).await;
                }
            }
        }
    }
}
