use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;

use crate::mesh::types::MeshSignal;
use crate::mesh::expert::{ExpertMsg, WorkEnvelope, WorkPayload, ExpertState, BatonPass, ExpertError};

const RUST_CAPABILITIES: &[&str] = &["rust", "memory", "thread", "async", "tokio"];
const MAX_DELEGATION_DEPTH: u32 = 3;

pub struct RustExpert {
    base: ExpertState,
    domain_knowledge: HashMap<String, String>,
    processed_count: u64,
}

impl RustExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "RustExpert".to_string(),
            RUST_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut domain_knowledge = HashMap::new();
        domain_knowledge.insert("ownership".to_string(), "Rust's ownership system ensures memory safety without GC".to_string());
        domain_knowledge.insert("borrowing".to_string(), "References must always be valid".to_string());
        domain_knowledge.insert("lifetimes".to_string(), "Lifetimes ensure references remain valid".to_string());
        domain_knowledge.insert("async".to_string(), "Rust async uses futures and the await keyword".to_string());

        Self {
            base,
            domain_knowledge,
            processed_count: 0,
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        let mut expert = Self::new();
        expert.base.timeout_secs = timeout_secs;
        expert
    }

    fn can_handle_locally(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        for key in self.domain_knowledge.keys() {
            if query_lower.contains(key) {
                return true;
            }
        }
        query_lower.contains("rust") || query_lower.contains("cargo")
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        self.processed_count += 1;

        let query = match &envelope.payload {
            WorkPayload::Query(q) => q.to_lowercase(),
            WorkPayload::Process(p) => p.to_lowercase(),
            WorkPayload::Delegate { capability, .. } => {
                return format!("RustExpert: Received delegation for {}", capability);
            }
        };

        for (key, value) in &self.domain_knowledge {
            if query.contains(key) {
                return format!("RustExpert: {}: {}", key, value);
            }
        }

        if query.contains("thread") {
            return "RustExpert: Rust uses 1:1 threading with std::thread and async tasks via executors".to_string();
        }

        format!("RustExpert: Processed query #{} (awaiting peer if needed)", self.processed_count)
    }

    fn should_delegate(&self, query: &str, hop_count: u32) -> Option<String> {
        if hop_count >= MAX_DELEGATION_DEPTH {
            return None;
        }

        let query_lower = query.to_lowercase();

        if query_lower.contains("frontend") || query_lower.contains("ui") || query_lower.contains("react") {
            return Some("frontend".to_string());
        }

        if query_lower.contains("database") || query_lower.contains("sql") || query_lower.contains("query") {
            return Some("database".to_string());
        }

        None
    }

    fn handle_delegation(
        &mut self,
        envelope: &WorkEnvelope,
        target_capability: String,
        myself: &ActorRef<ExpertMsg>,
    ) -> Result<(), ExpertError> {
        let pending_task = BatonPass::create_pending_task(envelope, target_capability.clone());

        tracing::info!(
            trace_id = %envelope.trace_id,
            from = %self.base.name,
            to = %target_capability,
            "Delegating to peer capability"
        );

        self.base.store_pending_task(pending_task);

        Err(ExpertError::CapabilityNotFound(format!("Delegated to {}", target_capability)))
    }
}

#[async_trait::async_trait]
impl Actor for RustExpert {
    type Msg = ExpertMsg;
    type State = RustExpert;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::new())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ExpertMsg::Work(envelope) => {
                state.base.cleanup_expired_tasks();

                if let Err(e) = BatonPass::check_hop_limit(envelope.hop_count) {
                    tracing::warn!(
                        trace_id = %envelope.trace_id,
                        error = %e,
                        "Hop limit exceeded"
                    );
                    if let Some(reply_to) = &envelope.reply_to {
                        let _ = reply_to.cast(ExpertMsg::PeerResponse {
                            trace_id: envelope.trace_id,
                            result: format!("Error: {}", e),
                        });
                    }
                    return Ok(());
                }

                if let Some(delegation_target) = state.should_delegate(
                    match &envelope.payload {
                        WorkPayload::Query(q) => q.as_str(),
                        WorkPayload::Process(p) => p.as_str(),
                        WorkPayload::Delegate { capability, .. } => capability.as_str(),
                    },
                    envelope.hop_count,
                ) {
                    let _ = state.handle_delegation(&envelope, delegation_target, &_myself);
                } else if state.can_handle_locally(match &envelope.payload {
                    WorkPayload::Query(q) => q.as_str(),
                    WorkPayload::Process(p) => p.as_str(),
                    WorkPayload::Delegate { .. } => "",
                }) {
                    let result = state.process_locally(&envelope);

                    if let Some(reply_to) = &envelope.reply_to {
                        let _ = reply_to.cast(ExpertMsg::PeerResponse {
                            trace_id: envelope.trace_id,
                            result,
                        });
                    }

                    tracing::info!(
                        trace_id = %envelope.trace_id,
                        expert = %state.base.name,
                        "Processed locally"
                    );
                } else {
                    let result = format!("{}: Query understood but no direct answer available", state.base.name);

                    if let Some(reply_to) = &envelope.reply_to {
                        let _ = reply_to.cast(ExpertMsg::PeerResponse {
                            trace_id: envelope.trace_id,
                            result,
                        });
                    }
                }
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(_task) = state.base.pending_tasks.remove(&trace_id) {
                    tracing::info!(
                        trace_id = %trace_id,
                        peer_result = %result,
                        "Peer response received, resuming processing"
                    );

                    let final_result = format!("{} (peer-assisted): {}", state.base.name, result);

                    if let Some(task) = state.base.pending_tasks.get(&trace_id) {
                        if let Some(reply_to) = &task.reply_to {
                            let _ = reply_to.cast(ExpertMsg::PeerResponse {
                                trace_id,
                                result: final_result,
                            });
                        }
                    }
                }
            }
            ExpertMsg::MeshSignal(signal) => match signal {
                MeshSignal::Cancel => {
                    tracing::info!(
                        expert = %state.base.name,
                        pending = state.base.pending_tasks.len(),
                        "Cancel signal received"
                    );
                    state.base.pending_tasks.clear();
                }
            },
        }
        Ok(())
    }
}

pub async fn spawn_rust_expert() -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, RustExpert::new(), ()).await?;
    Ok(actor_ref)
}

pub async fn spawn_rust_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, RustExpert::with_timeout(timeout_secs), ()).await?;
    Ok(actor_ref)
}
