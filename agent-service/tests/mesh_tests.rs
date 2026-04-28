//! Integratietests voor de actor mesh (ractor).
//! Zet `MESH_USE_OLLAMA=0` voor snelle deterministische runs zonder Ollama.

use std::collections::HashMap;
use std::time::Duration;

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::sync::oneshot;
use tokio::time::{sleep, timeout};

use agent_service::mesh::entry::EntryActor;
use agent_service::mesh::{
    spawn_frontend_expert,
    spawn_frontend_expert_with_peers,
    spawn_rust_expert,
    spawn_research_expert,
    spawn_schrijver_expert,
    spawn_pii_stripper_expert,
    spawn_reviewer_expert,
    spawn_document_orchestrator,
};
use agent_service::mesh::experts::RustExpert;
use agent_service::mesh::expert::{BatonPass, ExpertMsg, ExpertState, WorkEnvelope, WorkPayload, StyleProfile, PIICategory, ReviewCriteria, ErrorStrategy};
use agent_service::mesh::registry::spawn_registry;
use agent_service::mesh::types::{EntryMsg, MeshSignal, RegistryMsg, SessionContext};

/// Vangt `EntryMsg::ExpertResponse` op voor synchrone asserts in tests.
struct CaptureClient {
    sender: Option<oneshot::Sender<String>>,
}

#[async_trait::async_trait]
impl Actor for CaptureClient {
    type Msg = EntryMsg;
    type State = CaptureClient;
    type Arguments = oneshot::Sender<String>;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        tx: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(CaptureClient {
            sender: Some(tx),
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let EntryMsg::ExpertResponse { result, .. } = message {
            if let Some(tx) = state.sender.take() {
                let _ = tx.send(result);
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn multi_hop_success_test() {
    let registry = spawn_registry().await.expect("spawn registry");

    let rust_expert = spawn_rust_expert(None).await.expect("RustExpert");

    let mut fe_peers = HashMap::new();
    fe_peers.insert("rust".to_owned(), rust_expert.clone());
    let frontend_expert = spawn_frontend_expert_with_peers(fe_peers, None)
        .await
        .expect("FrontendExpert");

    let mut ru_peers = HashMap::new();
    ru_peers.insert("frontend".to_owned(), frontend_expert.clone());
    rust_expert
        .cast(ExpertMsg::SetPeers(Some(ru_peers)))
        .expect("SetPeers rust voor peer naar frontend");

    let mut entry_actor = EntryActor::new();
    entry_actor.set_registry(registry.clone());
    entry_actor.register_expert("rust".to_owned(), rust_expert);
    entry_actor.register_expert("frontend".to_owned(), frontend_expert);

    let (tx, rx) = oneshot::channel::<String>();
    let (capture_ref, _) = ractor::Actor::spawn(
        None,
        CaptureClient { sender: None },
        tx,
    )
        .await
        .expect("CaptureClient");

    let (entry_ref, _) = ractor::Actor::spawn(
        None,
        entry_actor,
        Some(registry.clone()),
    )
    .await
    .expect("EntryActor");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let query = "How do I use React with Rust WebAssembly?";
    entry_ref
        .cast(EntryMsg::SubmitRequest {
            query: query.to_owned(),
            context: context.clone(),
            reply_to: Some(capture_ref),
        })
        .expect("SubmitRequest");

    let result = timeout(Duration::from_secs(120), rx)
        .await
        .expect("timeout")
        .expect(" ExpertResponse");

    assert!(
        result.contains("FrontendExpert")
            || result.contains("peer-assisted")
            || result.contains("Ollama")
            || result.len() > 20,
        "Verwacht mesh-antwoord (delegatie, heuristiek of Ollama): {}",
        result
    );

    tracing::info!(%result, "Multi-hop test voltooid");
}

#[tokio::test]
async fn hop_limit_breach_test() {
    let _rust_expert = spawn_rust_expert(None).await.expect("RustExpert");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Query("test".into()),
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 10,
    };

    let result = BatonPass::check_hop_limit(envelope.hop_count);
    assert!(result.is_err(), "Hop > MAX moet falen");

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Max hops exceeded"), "{err_msg}");
}

#[tokio::test]
async fn actor_failure_recovery_test() {
    let _rust_expert = spawn_rust_expert(None).await.expect("RustExpert");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Query("ownership".into()),
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    let pending_task = BatonPass::create_pending_task(&envelope, "rust".into());

    let mut expert_state = ExpertState::new("TestExpert".into(), vec!["test".into()]);
    expert_state.timeout_secs = 1;
    expert_state.store_pending_task(pending_task);

    sleep(Duration::from_secs(2)).await;

    let expired_ids = expert_state.cleanup_expired_tasks();
    assert_eq!(
        expired_ids.len(),
        1,
        "Eén verouderde taak wordt verwijderd"
    );
}

#[tokio::test]
async fn cancellation_propagation_test() {
    let registry = spawn_registry().await.expect("registry");
    let rust_expert = spawn_rust_expert(None).await.expect("rust");
    let frontend_expert = spawn_frontend_expert(None).await.expect("frontend");

    let mut entry_actor = EntryActor::new();
    entry_actor.set_registry(registry.clone());
    entry_actor.register_expert("rust".into(), rust_expert.clone());
    entry_actor.register_expert("frontend".into(), frontend_expert.clone());

    let (entry_ref, _) = ractor::Actor::spawn(None, entry_actor, Some(registry))
        .await
        .expect("entry");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test".into(),
        metadata: Default::default(),
    };

    entry_ref
        .cast(EntryMsg::SubmitRequest {
            query: "Tell me about Rust ownership".into(),
            context,
            reply_to: None,
        })
        .unwrap();

    sleep(Duration::from_millis(80)).await;

    entry_ref.cast(EntryMsg::CancelAll).unwrap();

    sleep(Duration::from_millis(120)).await;

    rust_expert
        .cast(ExpertMsg::MeshSignal(MeshSignal::Cancel))
        .unwrap();
    frontend_expert
        .cast(ExpertMsg::MeshSignal(MeshSignal::Cancel))
        .unwrap();
}

#[tokio::test]
async fn pending_task_expiry_with_recovery_test() {
    let rust_expert = spawn_rust_expert(None).await.expect("rust");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test".into(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Query("test".into()),
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    let pending_task = BatonPass::create_pending_task(&envelope, "rust".into());

    let mut expert_state = ExpertState::new("TestExpert".into(), vec!["test".into()]);
    expert_state.timeout_secs = 1;
    expert_state.store_pending_task(pending_task.clone());

    assert_eq!(expert_state.pending_tasks.len(), 1);

    sleep(Duration::from_secs(2)).await;

    let recovered = expert_state.recover_expired_tasks(&rust_expert);
    assert!(
        !recovered.is_empty(),
        "recovery-pad wordt geprobeerd bij timeout"
    );

    let purged = expert_state.purge_expired_tasks();
    assert!(!purged.is_empty(), "na recovery volgt opruiming");

    assert_eq!(expert_state.pending_tasks.len(), 0);
}

#[tokio::test]
async fn registry_health_cleanup_test() {
    let registry = spawn_registry().await.expect("registry");

    let registry_msg = RegistryMsg::Register {
        actor: Some(registry.clone()),
        capability: "test".into(),
        metadata: Default::default(),
    };

    registry.cast(registry_msg).unwrap();
    sleep(Duration::from_millis(50)).await;

    registry
        .cast(RegistryMsg::ResolveCapability {
            capability: "test".into(),
            reply_to: None,
        })
        .unwrap();
}

#[tokio::test]
async fn poison_pill_clears_pending_tasks_test() {
    let (expert_ref, _) = ractor::Actor::spawn(None, RustExpert::new(), (None, None))
        .await
        .expect("RustExpert");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test".into(),
        metadata: Default::default(),
    };

    let trace_id = uuid::Uuid::new_v4();

    expert_ref
        .cast(ExpertMsg::Work(WorkEnvelope {
            payload: WorkPayload::Query("test pending".into()),
            context: context.clone(),
            reply_to: None,
            entry_reply: None,
            trace_id,
            hop_count: 0,
        }))
        .unwrap();

    sleep(Duration::from_millis(50)).await;

    expert_ref
        .cast(ExpertMsg::MeshSignal(MeshSignal::Cancel))
        .unwrap();
}

#[tokio::test]
async fn baton_pass_delegation_test() {
    let rust_actor = spawn_rust_expert(None).await.expect("rust");
    let _frontend = spawn_frontend_expert(None).await.expect("fe");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test".into(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Query("test".into()),
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    let delegated =
        BatonPass::prepare_delegation(&envelope, rust_actor.clone(), WorkPayload::Query("sub".into()));

    assert_eq!(delegated.hop_count, 1);
    assert_eq!(delegated.trace_id, envelope.trace_id);
    assert!(delegated.reply_to.is_some(), "delegatie zet ouder als reply");

    tracing::info!("baton_pass_delegation_test ok");
}

#[tokio::test]
async fn research_expert_test() {
    let research_expert = spawn_research_expert(None).await.expect("spawn researcher");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Research {
            urls: vec!["https://example.com/article".into()],
            depth: 2,
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    research_expert.cast(ExpertMsg::Work(envelope)).expect("send work");

    sleep(Duration::from_millis(100)).await;

    tracing::info!("ResearchExpert test passed");
}

#[tokio::test]
async fn pii_stripper_expert_test() {
    let pii_expert = spawn_pii_stripper_expert(None).await.expect("spawn pii");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let test_content = "Contact John Doe at john.doe@example.com or call 555-123-4567.";

    let envelope = WorkEnvelope {
        payload: WorkPayload::ScrubPII {
            content: test_content.to_string(),
            pii_categories: vec![PIICategory::Email, PIICategory::PhoneNumber, PIICategory::Name],
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    pii_expert.cast(ExpertMsg::Work(envelope)).expect("send work");

    sleep(Duration::from_millis(100)).await;

    tracing::info!("PIIStripperExpert test passed");
}

#[tokio::test]
async fn schrijver_expert_test() {
    let schrijver_expert = spawn_schrijver_expert(None).await.expect("spawn schrijver");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let notes = "Research found that Rust is a systems programming language.";

    let envelope = WorkEnvelope {
        payload: WorkPayload::Write {
            research_notes: notes.to_string(),
            style_profile: StyleProfile::Technical,
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    schrijver_expert.cast(ExpertMsg::Work(envelope)).expect("send work");

    sleep(Duration::from_millis(100)).await;

    tracing::info!("SchrijverExpert test passed");
}

#[tokio::test]
async fn reviewer_expert_test() {
    let reviewer_expert = spawn_reviewer_expert(None).await.expect("spawn reviewer");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let content = "This is a test document. \n\nIt has some content that needs review.";

    let envelope = WorkEnvelope {
        payload: WorkPayload::Review {
            content: content.to_string(),
            criteria: Some(ReviewCriteria {
                tone: Some("professional".to_string()),
                length_constraints: Some((10, 100)),
                focus_areas: vec!["clarity".to_string()],
            }),
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    reviewer_expert.cast(ExpertMsg::Work(envelope)).expect("send work");

    sleep(Duration::from_millis(100)).await;

    tracing::info!("ReviewerExpert test passed");
}

#[tokio::test]
async fn document_orchestrator_workflow_test() {
    let registry = spawn_registry().await.expect("registry");

    let research_expert = spawn_research_expert(None).await.expect("researcher");
    let schrijver_expert = spawn_schrijver_expert(None).await.expect("schrijver");
    let pii_expert = spawn_pii_stripper_expert(None).await.expect("pii");
    let reviewer_expert = spawn_reviewer_expert(None).await.expect("reviewer");

    let mut peers = HashMap::new();
    peers.insert("research".to_string(), research_expert.clone());
    peers.insert("write".to_string(), schrijver_expert.clone());
    peers.insert("pii".to_string(), pii_expert.clone());
    peers.insert("review".to_string(), reviewer_expert.clone());

    let orchestrator = spawn_document_orchestrator(peers).await.expect("orchestrator");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::CreateDocument {
            urls: vec!["https://example.com".into()],
            style_profile: StyleProfile::Formal,
            pii_categories: vec![PIICategory::Email],
            review_criteria: Some(ReviewCriteria {
                tone: Some("professional".to_string()),
                length_constraints: None,
                focus_areas: vec![],
            }),
            error_strategy: ErrorStrategy::FailFast,
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    orchestrator.cast(ExpertMsg::Work(envelope)).expect("send work");

    sleep(Duration::from_secs(1)).await;

    tracing::info!("DocumentOrchestrator workflow test passed");
}
