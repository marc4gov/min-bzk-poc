//! Één mesh-run voor HTTP/Tauri. Gebruikt optioneel Ollama (`MESH_USE_OLLAMA`, `OLLAMA_*`).

use std::collections::HashMap;

use ractor::{Actor, ActorProcessingErr, ActorRef};
use serde::Serialize;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::time::Duration;
use uuid::Uuid;

use crate::mesh::document_improver::spawn_document_improver;
use crate::mesh::document_orchestrator::spawn_document_orchestrator;
use crate::mesh::entry::{triage_entry_capability, EntryActor};
use crate::mesh::expert::{
    ErrorStrategy, ExpertMsg, PIICategory, PeerMap, StyleProfile, WorkEnvelope, WorkPayload,
};
use crate::mesh::experts::{
    spawn_frontend_expert_with_peers, spawn_pii_stripper_expert, spawn_research_expert,
    spawn_reviewer_expert, spawn_rust_expert, spawn_schrijver_expert,
};
use crate::mesh::research_http;
use crate::mesh::{spawn_registry, EntryMsg, RegistryMsg, SessionContext};

/// Antwoord naar de UI (`/api/mesh/demo`, `/api/mesh/demo/document`).
#[derive(Debug, Serialize)]
pub struct MeshDemoResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

struct CaptureEntryReply {
    tx: Option<oneshot::Sender<String>>,
}

#[async_trait::async_trait]
impl Actor for CaptureEntryReply {
    type Msg = EntryMsg;
    type State = CaptureEntryReply;
    type Arguments = oneshot::Sender<String>;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(CaptureEntryReply { tx: Some(args) })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let EntryMsg::ExpertResponse { result, .. } = message {
            if let Some(sender) = state.tx.take() {
                let _ = sender.send(result);
            }
        }
        Ok(())
    }
}

/// Voert één SubmitRequest langs Entry naar experts uit en wacht op `ExpertResponse`.
///
/// `full_mesh=true`: volledige topology; triage **`document`** → direct **CreateDocument**-pipeline ([`mesh_run_document_pipeline`]) met URL’s uit de tekst of `https://example.org`.
/// `full_mesh=false`: klassiek Rust+Frontend.
///
/// Bij `live_tx` worden mesh-stappen als [`crate::AgentEvent::Mesh`] uitgezonden naar WebSocket‑clients.
pub async fn mesh_run_demo_query(
    query: &str,
    timeout_secs: u64,
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
    full_mesh: bool,
) -> MeshDemoResult {
    let q = query.trim();
    if q.is_empty() {
        return MeshDemoResult {
            ok: false,
            result: None,
            error: Some("Lege query".into()),
        };
    }

    // Langere grens voor volledige mesh + document‑pipeline (meerdere actors / HTTP‑research).
    let wait = timeout_secs.max(5).min(600);

    crate::mesh::live::emit_mesh(
        &live_tx,
        "mesh",
        "sessie",
        &format!("start: {}", preview_query(q)),
    );

    let run_fut = if full_mesh {
        run_mesh_full_once(q, wait, live_tx.clone()).await
    } else {
        run_mesh_once(q, wait, live_tx.clone()).await
    };

    match run_fut {
        Ok(s) => {
            crate::mesh::live::emit_mesh(&live_tx, "mesh", "klaar", &preview_query(&s));
            MeshDemoResult {
                ok: true,
                result: Some(s),
                error: None,
            }
        }
        Err(e) => {
            crate::mesh::live::emit_mesh(&live_tx, "mesh", "fout", &e);
            MeshDemoResult {
                ok: false,
                result: None,
                error: Some(e),
            }
        }
    }
}

/// DocumentOrchestrator: research → schrijven → PII scrub → review (`CreateDocument`).
/// Ook bereikbaar vanuit **`mesh_run_demo_query`** bij `full_mesh` + triage **document** ([`crate::mesh::triage_entry_capability`]).
pub async fn mesh_run_document_pipeline(
    urls: Vec<String>,
    timeout_secs: u64,
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
) -> MeshDemoResult {
    if urls.is_empty() {
        return MeshDemoResult {
            ok: false,
            result: None,
            error: Some("Minstens één URL vereist".into()),
        };
    }

    let wait = timeout_secs.max(15).min(600);

    crate::mesh::live::emit_mesh(
        &live_tx,
        "mesh",
        "document_sessie",
        &format!("{} URL(s)", urls.len(),),
    );

    match run_document_pipeline_inner(urls, wait, live_tx.clone()).await {
        Ok(s) => {
            crate::mesh::live::emit_mesh(&live_tx, "mesh", "document_klaar", &preview_query(&s));
            MeshDemoResult {
                ok: true,
                result: Some(s),
                error: None,
            }
        }
        Err(e) => {
            crate::mesh::live::emit_mesh(&live_tx, "mesh", "document_fout", &e);
            MeshDemoResult {
                ok: false,
                result: None,
                error: Some(e),
            }
        }
    }
}

/// DocumentImprover: analyseer → verbeter → PII scrub voor geüploade documenten.
/// Wordt aangeroepen vanuit de Tauri frontend voor document upload + verbetering.
pub async fn mesh_improve_uploaded_document(
    content: String,
    filename: String,
    instructions: String,
    timeout_secs: u64,
    model: Option<String>,
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
) -> MeshDemoResult {
    if content.is_empty() {
        return MeshDemoResult {
            ok: false,
            result: None,
            error: Some("Lege document inhoud".into()),
        };
    }

    let wait = timeout_secs.max(15).min(600);

    crate::mesh::live::emit_mesh(
        &live_tx,
        "mesh",
        "improve_sessie",
        &format!("bestand: {}", filename),
    );

    match run_document_improver_inner(content, filename, instructions, wait, model, live_tx.clone()).await {
        Ok(s) => {
            crate::mesh::live::emit_mesh(&live_tx, "mesh", "improve_klaar", &preview_query(&s));
            MeshDemoResult {
                ok: true,
                result: Some(s),
                error: None,
            }
        }
        Err(e) => {
            crate::mesh::live::emit_mesh(&live_tx, "mesh", "improve_fout", &e);
            MeshDemoResult {
                ok: false,
                result: None,
                error: Some(e),
            }
        }
    }
}

fn preview_query(s: &str) -> String {
    let t = s.trim();
    let mut iter = t.chars();
    let head: String = iter.by_ref().take(120).collect();
    if iter.next().is_some() {
        head + "…"
    } else {
        head
    }
}

struct FullTopology {
    registry: ActorRef<RegistryMsg>,
    rust: ActorRef<ExpertMsg>,
    frontend: ActorRef<ExpertMsg>,
    research: ActorRef<ExpertMsg>,
    schrijver: ActorRef<ExpertMsg>,
    pii: ActorRef<ExpertMsg>,
    reviewer: ActorRef<ExpertMsg>,
    document: ActorRef<ExpertMsg>,
}

async fn assemble_full_topology(
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<FullTopology, String> {
    let registry = spawn_registry()
        .await
        .map_err(|e| format!("registry: {}", e))?;

    crate::mesh::live::emit_mesh(&live_tx, "mesh", "registry", "registry online");

    let research = spawn_research_expert(live_tx.clone())
        .await
        .map_err(|e| format!("research expert: {}", e))?;

    let schrijver = spawn_schrijver_expert(live_tx.clone())
        .await
        .map_err(|e| format!("schrijver expert: {}", e))?;

    let pii = spawn_pii_stripper_expert(live_tx.clone())
        .await
        .map_err(|e| format!("pii expert: {}", e))?;

    let reviewer = spawn_reviewer_expert(live_tx.clone())
        .await
        .map_err(|e| format!("reviewer expert: {}", e))?;

    let mut doc_peers = PeerMap::new();
    doc_peers.insert("research".into(), research.clone());
    doc_peers.insert("write".into(), schrijver.clone());
    doc_peers.insert("pii".into(), pii.clone());
    doc_peers.insert("review".into(), reviewer.clone());

    let document = spawn_document_orchestrator(doc_peers)
        .await
        .map_err(|e| format!("document orchestrator: {}", e))?;

    let rust = spawn_rust_expert(live_tx.clone())
        .await
        .map_err(|e| format!("rust expert: {}", e))?;

    let mut frontend_peers = PeerMap::new();
    frontend_peers.insert("rust".into(), rust.clone());
    let frontend = spawn_frontend_expert_with_peers(frontend_peers, live_tx.clone())
        .await
        .map_err(|e| format!("frontend expert: {}", e))?;

    // SQL/database-triage gebruikt géén eigen actor; delegatie gebruikt `"database"` als peer-key.
    let database_peer = research.clone();

    let mut rust_peers = PeerMap::new();
    rust_peers.insert("frontend".into(), frontend.clone());
    rust_peers.insert("database".into(), database_peer.clone());

    rust.cast(ExpertMsg::SetPeers(Some(rust_peers)))
        .map_err(|e| format!("rust SetPeers: {:?}", e))?;

    let mut fp = PeerMap::new();
    fp.insert("rust".into(), rust.clone());
    fp.insert("database".into(), database_peer);

    frontend
        .cast(ExpertMsg::SetPeers(Some(fp)))
        .map_err(|e| format!("frontend SetPeers: {:?}", e))?;

    crate::mesh::live::emit_mesh(
        &live_tx,
        "mesh",
        "experts",
        "volledige topology: Rust, Frontend, Research, Schrijver, PII, Reviewer, DocumentOrchestrator",
    );

    Ok(FullTopology {
        registry,
        rust,
        frontend,
        research,
        schrijver,
        pii,
        reviewer,
        document,
    })
}

fn register_full_entry(mesh: &FullTopology, entry: &mut EntryActor) {
    entry.register_expert("rust".into(), mesh.rust.clone());
    entry.register_expert("frontend".into(), mesh.frontend.clone());
    entry.register_expert("research".into(), mesh.research.clone());
    entry.register_expert("write".into(), mesh.schrijver.clone());
    entry.register_expert("pii".into(), mesh.pii.clone());
    entry.register_expert("review".into(), mesh.reviewer.clone());
    entry.register_expert("database".into(), mesh.research.clone());
    entry.register_expert("general".into(), mesh.frontend.clone());
}

async fn run_mesh_full_once(
    query: &str,
    wait_secs: u64,
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<String, String> {
    if triage_entry_capability(query) == "document" {
        crate::mesh::live::emit_mesh(&live_tx, "Entry", "triage_document", &preview_query(query));
        let mut urls = research_http::extract_http_urls_from_text(query);
        if urls.is_empty() {
            urls.push("https://example.org".into());
        }
        let doc_wait = wait_secs.max(15).min(600);
        return run_document_pipeline_inner(urls, doc_wait, live_tx).await;
    }

    let mesh = assemble_full_topology(live_tx.clone()).await?;

    let mut entry_actor = EntryActor::new();
    entry_actor.set_registry(mesh.registry.clone());
    register_full_entry(&mesh, &mut entry_actor);

    await_entry_response(
        query,
        wait_secs,
        live_tx,
        entry_actor,
        Some(mesh.registry.clone()),
    )
    .await
}

async fn await_entry_response(
    query: &str,
    wait_secs: u64,
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
    entry_actor: EntryActor,
    registry_opt: Option<ActorRef<RegistryMsg>>,
) -> Result<String, String> {
    let (tx, rx) = oneshot::channel::<String>();

    let (capture_ref, _) = ractor::Actor::spawn(None, CaptureEntryReply { tx: None }, tx)
        .await
        .map_err(|e| format!("capture actor: {:?}", e))?;

    let (entry_ref, _) = ractor::Actor::spawn(None, entry_actor, registry_opt)
        .await
        .map_err(|e| format!("entry actor: {:?}", e))?;

    let context = SessionContext {
        session_id: Uuid::new_v4(),
        user_id: "mesh_demo".into(),
        metadata: Default::default(),
    };

    crate::mesh::live::emit_mesh(&live_tx, "Entry", "SubmitRequest", &preview_query(query));

    entry_ref
        .cast(EntryMsg::SubmitRequest {
            query: query.to_owned(),
            context,
            reply_to: Some(capture_ref),
        })
        .map_err(|e| format!("SubmitRequest cast: {:?}", e))?;

    match tokio::time::timeout(Duration::from_secs(wait_secs), rx).await {
        Ok(Ok(s)) => Ok(s),
        Ok(Err(_)) => Err("Geen ExpertResponse ontvangen (kanaal gesloten).".into()),
        Err(_) => Err(format!(
            "Timeout na {}s — mesh gaf geen antwoord terug.",
            wait_secs
        )),
    }
}

async fn run_mesh_once(
    query: &str,
    wait_secs: u64,
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<String, String> {
    let registry = spawn_registry()
        .await
        .map_err(|e| format!("registry: {}", e))?;

    crate::mesh::live::emit_mesh(&live_tx, "mesh", "registry", "registry online");

    let rust = spawn_rust_expert(live_tx.clone())
        .await
        .map_err(|e| format!("rust expert: {}", e))?;

    let mut frontend_peers = HashMap::new();
    frontend_peers.insert("rust".into(), rust.clone());
    let frontend = spawn_frontend_expert_with_peers(frontend_peers, live_tx.clone())
        .await
        .map_err(|e| format!("frontend expert: {}", e))?;

    let mut rust_peers = HashMap::new();
    rust_peers.insert("frontend".into(), frontend.clone());
    rust.cast(ExpertMsg::SetPeers(Some(rust_peers)))
        .map_err(|e| format!("rust SetPeers (peer naar frontend): {:?}", e))?;

    crate::mesh::live::emit_mesh(
        &live_tx,
        "mesh",
        "experts",
        "RustExpert en FrontendExpert gekoppeld",
    );

    let mut entry_actor = EntryActor::new();
    entry_actor.set_registry(registry.clone());
    entry_actor.register_expert("rust".into(), rust);
    entry_actor.register_expert("frontend".into(), frontend);

    await_entry_response(query, wait_secs, live_tx, entry_actor, Some(registry)).await
}

async fn run_document_pipeline_inner(
    urls: Vec<String>,
    wait_secs: u64,
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<String, String> {
    let mesh = assemble_full_topology(live_tx.clone()).await?;

    let (tx, rx) = oneshot::channel::<String>();

    let (capture_ref, _) = ractor::Actor::spawn(None, CaptureEntryReply { tx: None }, tx)
        .await
        .map_err(|e| format!("capture actor: {:?}", e))?;

    let trace_id = Uuid::new_v4();
    let envelope = WorkEnvelope {
        payload: WorkPayload::CreateDocument {
            urls,
            style_profile: StyleProfile::Formal,
            pii_categories: vec![PIICategory::Email, PIICategory::Name],
            review_criteria: None,
            error_strategy: ErrorStrategy::FailFast,
        },
        context: SessionContext {
            session_id: Uuid::new_v4(),
            user_id: "mesh_document".into(),
            metadata: HashMap::new(),
        },
        reply_to: None,
        entry_reply: Some(capture_ref),
        trace_id,
        hop_count: 0,
    };

    crate::mesh::live::emit_mesh(
        &live_tx,
        "DocumentOrchestrator",
        "CreateDocument",
        &format!("trace={}", trace_id),
    );

    mesh.document
        .cast(ExpertMsg::Work(envelope))
        .map_err(|e| format!("Document Work cast: {:?}", e))?;

    match tokio::time::timeout(Duration::from_secs(wait_secs), rx).await {
        Ok(Ok(s)) => Ok(s),
        Ok(Err(_)) => Err("Geen ExpertResponse ontvangen (kanaal gesloten).".into()),
        Err(_) => Err(format!(
            "Timeout na {}s — document-pipeline niet afgerond.",
            wait_secs
        )),
    }
}

async fn run_document_improver_inner(
    content: String,
    filename: String,
    instructions: String,
    wait_secs: u64,
    model: Option<String>,
    live_tx: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<String, String> {
    // Spawn required experts
    let registry = spawn_registry()
        .await
        .map_err(|e| format!("registry: {}", e))?;

    crate::mesh::live::emit_mesh(&live_tx, "mesh", "registry", "registry online");

    let research = spawn_research_expert(live_tx.clone())
        .await
        .map_err(|e| format!("research expert: {}", e))?;
    let write = spawn_schrijver_expert(live_tx.clone())
        .await
        .map_err(|e| format!("schrijver expert: {}", e))?;
    let pii = spawn_pii_stripper_expert(live_tx.clone())
        .await
        .map_err(|e| format!("pii expert: {}", e))?;

    crate::mesh::live::emit_mesh(&live_tx, "mesh", "experts", "improver experts online");

    // Build peer map
    let mut peers = PeerMap::new();
    peers.insert("research".to_string(), research);
    peers.insert("write".to_string(), write);
    peers.insert("pii".to_string(), pii);

    // Spawn DocumentImprover
    let improver = spawn_document_improver(peers)
        .await
        .map_err(|e| format!("document improver: {}", e))?;

    crate::mesh::live::emit_mesh(&live_tx, "mesh", "improver", "DocumentImprover online");

    let (tx, rx) = oneshot::channel::<String>();

    let (capture_ref, _) = ractor::Actor::spawn(None, CaptureEntryReply { tx: None }, tx)
        .await
        .map_err(|e| format!("capture actor: {:?}", e))?;

    let trace_id = Uuid::new_v4();
    let envelope = WorkEnvelope {
        payload: WorkPayload::ImproveDocument {
            content,
            filename,
            instructions,
            pii_categories: vec![PIICategory::Email, PIICategory::Name, PIICategory::PhoneNumber],
        },
        context: SessionContext {
            session_id: Uuid::new_v4(),
            user_id: "improver".into(),
            metadata: HashMap::new(),
        },
        reply_to: None,
        entry_reply: Some(capture_ref),
        trace_id,
        hop_count: 0,
    };

    crate::mesh::live::emit_mesh(
        &live_tx,
        "DocumentImprover",
        "ImproveDocument",
        &format!("trace={}", trace_id),
    );

    improver
        .cast(ExpertMsg::Work(envelope))
        .map_err(|e| format!("Improver Work cast: {:?}", e))?;

    match tokio::time::timeout(Duration::from_secs(wait_secs), rx).await {
        Ok(Ok(s)) => Ok(s),
        Ok(Err(_)) => Err("Geen ExpertResponse ontvangen (kanaal gesloten).".into()),
        Err(_) => Err(format!(
            "Timeout na {}s — document-improver niet afgerond.",
            wait_secs
        )),
    }
}
