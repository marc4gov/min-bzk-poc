pub mod html_plain;
pub mod live;
pub mod ollama_bridge;
pub mod privacy_filter_bridge;
pub mod research_http;
pub mod run_once;

pub mod document_orchestrator;
pub mod document_improver;
pub mod entry;
pub mod expert;
pub mod experts;
pub mod registry;
pub mod types;

pub use document_orchestrator::{spawn_document_orchestrator, DocumentOrchestrator};
pub use document_improver::{spawn_document_improver, DocumentImprover};
pub use entry::triage_entry_capability;
pub use entry::{spawn_entry_actor, spawn_entry_actor_with_timeout, EntryActor, PendingRequest};
pub use expert::{
    spawn_expert, BatonPass, ErrorStrategy, ExpertError, ExpertMsg, ExpertState, PIICategory,
    PeerMap, PendingTask, ReviewCriteria, StyleProfile, TaskState, WorkEnvelope, WorkPayload,
    WorkflowState, WorkflowStep, MAX_DELEGATION_DEPTH,
};
pub use experts::{
    spawn_frontend_expert, spawn_frontend_expert_with_peers, spawn_frontend_expert_with_timeout,
    spawn_pii_stripper_expert, spawn_pii_stripper_expert_with_peers,
    spawn_pii_stripper_expert_with_timeout, spawn_research_expert,
    spawn_research_expert_with_peers, spawn_research_expert_with_timeout, spawn_reviewer_expert,
    spawn_reviewer_expert_with_peers, spawn_reviewer_expert_with_timeout, spawn_rust_expert,
    spawn_rust_expert_with_peers, spawn_rust_expert_with_timeout, spawn_schrijver_expert,
    spawn_schrijver_expert_with_peers, spawn_schrijver_expert_with_timeout, FrontendExpert,
    PIIStripperExpert, ResearchExpert, ReviewerExpert, RustExpert, SchrijverExpert,
};
pub use ollama_bridge::{chat_via_ollama_for_mesh, chat_via_ollama_for_mesh_with_events, mesh_ollama_enabled, set_mesh_model};
pub use privacy_filter_bridge::{privacy_model_scrub_enabled, ENV_PII_PRIVACY_SERVICE};
pub use registry::{spawn_registry, RegistryActor};
pub use research_http::{extract_http_urls_from_text, ENV_RESEARCH_HTTP};
pub use run_once::{mesh_improve_uploaded_document, mesh_run_demo_query, mesh_run_document_pipeline, MeshDemoResult};
pub use types::{EntryMsg, Envelope, MeshSignal, RegistryMsg, SessionContext};
