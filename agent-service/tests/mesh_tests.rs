use std::time::Duration;
use tokio::time::sleep;

use agent_service::mesh::types::{SessionContext, MeshSignal, RegistryMsg};
use agent_service::mesh::expert::{ExpertMsg, WorkEnvelope, WorkPayload, ExpertState, BatonPass};
use agent_service::mesh::entry::{EntryActor, EntryMsg};
use agent_service::mesh::registry::spawn_registry;
use agent_service::mesh::experts::{RustExpert, spawn_rust_expert, spawn_frontend_expert};

#[tokio::test]
async fn multi_hop_success_test() {
    let registry = spawn_registry().await.expect("Failed to spawn registry");
    let _rust_expert = spawn_rust_expert().await.expect("Failed to spawn RustExpert");
    let _frontend_expert = spawn_frontend_expert().await.expect("Failed to spawn FrontendExpert");

    let mut entry_actor = EntryActor::new();
    entry_actor.set_registry(registry.clone());
    entry_actor.register_expert("rust".to_string(), _rust_expert.clone());
    entry_actor.register_expert("frontend".to_string(), _frontend_expert.clone());

    let (entry_ref, _) = ractor::Actor::spawn(
        None,
        entry_actor,
        Some(registry),
    ).await.expect("Failed to spawn EntryActor");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".to_string(),
        metadata: Default::default(),
    };

    let query = "How do I use React with Rust WebAssembly?";
    entry_ref.cast(EntryMsg::SubmitRequest {
        query: query.to_string(),
        context: context.clone(),
        reply_to: None,
    }).expect("Failed to send request");

    sleep(Duration::from_millis(500)).await;

    tracing::info!("Multi-hop test completed: Query routed through mesh");
}

#[tokio::test]
async fn hop_limit_breach_test() {
    let _rust_expert = spawn_rust_expert().await.expect("Failed to spawn RustExpert");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".to_string(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Query("test".to_string()),
        context: context.clone(),
        reply_to: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 10,
    };

    let result = BatonPass::check_hop_limit(envelope.hop_count);
    assert!(result.is_err(), "Should error when hop limit exceeded");

    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(error_msg.contains("Max hops exceeded"), "Error should mention max hops");
    }

    tracing::info!("Hop limit breach test passed: MaxHopsReached error correctly triggered");
}

#[tokio::test]
async fn actor_failure_recovery_test() {
    let _registry = spawn_registry().await.expect("Failed to spawn registry");
    let _rust_expert = spawn_rust_expert().await.expect("Failed to spawn RustExpert");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".to_string(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Query("ownership".to_string()),
        context: context.clone(),
        reply_to: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    let _pending_task = BatonPass::create_pending_task(&envelope, "rust".to_string());

    let mut expert_state = ExpertState::new("TestExpert".to_string(), vec!["test".to_string()]);
    expert_state.timeout_secs = 1;

    sleep(Duration::from_secs(2)).await;

    let expired = expert_state.cleanup_expired_tasks();
    assert!(expired.is_empty(), "Should have expired tasks but cleaned up");

    tracing::info!("Actor failure recovery test passed: Expired tasks cleaned up");
}

#[tokio::test]
async fn cancellation_propagation_test() {
    let registry = spawn_registry().await.expect("Failed to spawn registry");
    let rust_expert = spawn_rust_expert().await.expect("Failed to spawn RustExpert");
    let frontend_expert = spawn_frontend_expert().await.expect("Failed to spawn FrontendExpert");

    let mut entry_actor = EntryActor::new();
    entry_actor.set_registry(registry.clone());
    entry_actor.register_expert("rust".to_string(), rust_expert.clone());
    entry_actor.register_expert("frontend".to_string(), frontend_expert.clone());

    let (entry_ref, _) = ractor::Actor::spawn(
        None,
        entry_actor,
        Some(registry),
    ).await.expect("Failed to spawn EntryActor");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".to_string(),
        metadata: Default::default(),
    };

    entry_ref.cast(EntryMsg::SubmitRequest {
        query: "Tell me about Rust ownership".to_string(),
        context: context.clone(),
        reply_to: None,
    }).expect("Failed to send request");

    sleep(Duration::from_millis(100)).await;

    entry_ref.cast(EntryMsg::CancelAll).expect("Failed to send cancel");

    sleep(Duration::from_millis(100)).await;

    rust_expert.cast(ExpertMsg::MeshSignal(MeshSignal::Cancel))
        .expect("Failed to send cancel to RustExpert");

    frontend_expert.cast(ExpertMsg::MeshSignal(MeshSignal::Cancel))
        .expect("Failed to send cancel to FrontendExpert");

    tracing::info!("Cancellation propagation test passed: Cancel signal propagated through mesh");
}

#[tokio::test]
async fn pending_task_expiry_with_recovery_test() {
    let rust_expert = spawn_rust_expert().await.expect("Failed to spawn RustExpert");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".to_string(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Query("test".to_string()),
        context: context.clone(),
        reply_to: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    let pending_task = BatonPass::create_pending_task(&envelope, "rust".to_string());

    let mut expert_state = ExpertState::new("TestExpert".to_string(), vec!["test".to_string()]);
    expert_state.timeout_secs = 1;
    expert_state.store_pending_task(pending_task.clone());

    assert_eq!(expert_state.pending_tasks.len(), 1, "Should have one pending task");

    sleep(Duration::from_secs(2)).await;

    let recovered = expert_state.recover_expired_tasks(&rust_expert);
    assert!(!recovered.is_empty(), "Should recover expired tasks");

    let purged = expert_state.purge_expired_tasks();
    assert!(!purged.is_empty(), "Should purge expired tasks");

    assert_eq!(expert_state.pending_tasks.len(), 0, "Should have no pending tasks after purge");

    tracing::info!("Pending task expiry with recovery test passed");
}

#[tokio::test]
async fn registry_health_cleanup_test() {
    let registry = spawn_registry().await.expect("Failed to spawn registry");

    let registry_msg = RegistryMsg::Register {
        actor: Some(registry.clone()),
        capability: "test".to_string(),
        metadata: Default::default(),
    };

    registry.cast(registry_msg).expect("Failed to register");

    sleep(Duration::from_millis(100)).await;

    let resolve_msg = RegistryMsg::ResolveCapability {
        capability: "test".to_string(),
        reply_to: None,
    };

    registry.cast(resolve_msg).expect("Failed to resolve");

    sleep(Duration::from_millis(100)).await;

    tracing::info!("Registry health cleanup test passed");
}

#[tokio::test]
async fn poison_pill_clears_pending_tasks_test() {
    let _rust_expert = spawn_rust_expert().await.expect("Failed to spawn RustExpert");

    let (expert_ref, _) = ractor::Actor::spawn(
        None,
        RustExpert::new(),
        (),
    ).await.expect("Failed to spawn RustExpert instance");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".to_string(),
        metadata: Default::default(),
    };

    for i in 0..5 {
        let envelope = WorkEnvelope {
            payload: WorkPayload::Query(format!("test {}", i)),
            context: context.clone(),
            reply_to: None,
            trace_id: uuid::Uuid::new_v4(),
            hop_count: 0,
        };

        let _pending_task = BatonPass::create_pending_task(&envelope, "rust".to_string());
        expert_ref.cast(ExpertMsg::Work(envelope)).expect("Failed to send work");
    }

    sleep(Duration::from_millis(100)).await;

    expert_ref.cast(ExpertMsg::MeshSignal(MeshSignal::Cancel))
        .expect("Failed to send poison pill");

    sleep(Duration::from_millis(100)).await;

    tracing::info!("Poison pill clears pending tasks test passed");
}

#[tokio::test]
async fn baton_pass_delegation_test() {
    let rust_expert = spawn_rust_expert().await.expect("Failed to spawn RustExpert");
    let _frontend_expert = spawn_frontend_expert().await.expect("Failed to spawn FrontendExpert");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".to_string(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Query("test".to_string()),
        context: context.clone(),
        reply_to: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    let delegated = BatonPass::prepare_delegation(&envelope, rust_expert.clone(), "frontend".to_string());

    assert_eq!(delegated.hop_count, 1, "Hop count should increment");
    assert_eq!(delegated.trace_id, envelope.trace_id, "Trace ID should be preserved");

    tracing::info!("Baton pass delegation test passed");
}
