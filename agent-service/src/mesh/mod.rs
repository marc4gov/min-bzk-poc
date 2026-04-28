pub mod live;
pub mod ollama_bridge;
pub mod run_once;

pub mod types;
pub mod registry;
pub mod expert;
pub mod entry;
pub mod experts;
pub mod document_orchestrator;

pub use types::{
    EntryMsg, Envelope, MeshSignal, RegistryMsg, SessionContext,
};
pub use ollama_bridge::{mesh_ollama_enabled, chat_via_ollama_for_mesh};
pub use run_once::{mesh_run_demo_query, MeshDemoResult};
pub use registry::{RegistryActor, spawn_registry};
pub use expert::{
    BatonPass,
    ExpertError,
    ExpertMsg,
    ExpertState,
    PeerMap,
    PendingTask,
    TaskState,
    WorkEnvelope,
    WorkPayload,
    spawn_expert,
    StyleProfile,
    PIICategory,
    ReviewCriteria,
    ErrorStrategy,
    WorkflowState,
    WorkflowStep,
};
pub use entry::{EntryActor, PendingRequest, spawn_entry_actor, spawn_entry_actor_with_timeout};
pub use experts::{
    FrontendExpert, RustExpert, ResearchExpert, SchrijverExpert, PIIStripperExpert, ReviewerExpert,
    spawn_frontend_expert, spawn_frontend_expert_with_peers,
    spawn_frontend_expert_with_timeout, spawn_rust_expert, spawn_rust_expert_with_peers,
    spawn_rust_expert_with_timeout,
    spawn_research_expert, spawn_research_expert_with_peers, spawn_research_expert_with_timeout,
    spawn_schrijver_expert, spawn_schrijver_expert_with_peers, spawn_schrijver_expert_with_timeout,
    spawn_pii_stripper_expert, spawn_pii_stripper_expert_with_peers, spawn_pii_stripper_expert_with_timeout,
    spawn_reviewer_expert, spawn_reviewer_expert_with_peers, spawn_reviewer_expert_with_timeout,
};
pub use document_orchestrator::{DocumentOrchestrator, spawn_document_orchestrator};
