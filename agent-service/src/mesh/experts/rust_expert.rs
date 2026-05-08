use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::expert::{
    send_work_output, BatonPass, ExpertError, ExpertMsg, ExpertState, PeerMap, WorkEnvelope,
    WorkPayload, MAX_DELEGATION_DEPTH,
};
use crate::mesh::types::{EntryMsg, MeshSignal};
use crate::mesh::ollama_bridge::chat_via_ollama_for_mesh_with_events;

const RUST_CAPABILITIES: &[&str] = &["rust", "memory", "thread", "async", "tokio"];

pub struct RustExpert {
    base: ExpertState,
    domain_knowledge: HashMap<String, String>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl RustExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "RustExpert".to_string(),
            RUST_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut domain_knowledge = HashMap::new();
        domain_knowledge.insert(
            "ownership".to_string(),
            "Rust's ownership system ensures memory safety without GC".to_string(),
        );
        domain_knowledge.insert(
            "borrowing".to_string(),
            "References must always be valid".to_string(),
        );
        domain_knowledge.insert(
            "lifetimes".to_string(),
            "Lifetimes ensure references remain valid".to_string(),
        );
        domain_knowledge.insert(
            "async".to_string(),
            "Rust async uses futures and the await keyword".to_string(),
        );

        Self {
            base,
            domain_knowledge,
            processed_count: 0,
            peers: None,
            mesh_events: None,
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        let mut expert = Self::new();
        expert.base.timeout_secs = timeout_secs;
        expert
    }

    pub fn with_peers(mut self, peers: PeerMap) -> Self {
        self.peers = Some(peers);
        self
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
            WorkPayload::Research { .. } => {
                return format!("RustExpert: Research request forwarded to ResearchExpert");
            }
            WorkPayload::Write { .. } => {
                return format!("RustExpert: Write request forwarded to SchrijverExpert");
            }
            WorkPayload::ScrubPII { .. } => {
                return format!("RustExpert: PII request forwarded to PIIStripperExpert");
            }
            WorkPayload::Review { .. } => {
                return format!("RustExpert: Review request forwarded to ReviewerExpert");
            }
            WorkPayload::CreateDocument { .. } => {
                return format!("RustExpert: Document creation forwarded to DocumentOrchestrator");
            }
            WorkPayload::ImproveDocument { .. } => {
                return format!("RustExpert: Document improvement forwarded to DocumentImprover");
            }
        };

        for (key, value) in &self.domain_knowledge {
            if query.contains(key) {
                return format!("RustExpert: {}: {}", key, value);
            }
        }

        if query.contains("thread") {
            return "RustExpert: Rust uses 1:1 threading with std::thread and async tasks via executors"
                .to_string();
        }

        format!(
            "RustExpert: Processed query #{} (awaiting peer if needed)",
            self.processed_count
        )
    }

    fn should_delegate(
        &self,
        query: &str,
        hop_count: u32,
        envelope: &WorkEnvelope,
    ) -> Option<String> {
        if hop_count >= MAX_DELEGATION_DEPTH {
            return None;
        }

        let query_lower = query.to_lowercase();

        let target = if query_lower.contains("frontend")
            || query_lower.contains("ui")
            || query_lower.contains("react")
        {
            Some("frontend".to_string())
        } else if query_lower.contains("database")
            || query_lower.contains("sql")
            || (query_lower.contains("query") && !query_lower.contains("react"))
        {
            Some("database".to_string())
        } else {
            None
        }?;

        if Self::would_delegate_to_sender(&self.peers, &target, envelope) {
            return None;
        }

        Some(target)
    }

    /// Delegatie niet terug naar dezelfde peer als `reply_to` (anders Rust↔Frontend ping-pong zonder einde).
    fn would_delegate_to_sender(
        peers: &Option<PeerMap>,
        target: &str,
        envelope: &WorkEnvelope,
    ) -> bool {
        match (
            peers.as_ref().and_then(|m| m.get(target)),
            envelope.reply_to.as_ref(),
        ) {
            (Some(peer), Some(reply)) => reply.get_id() == peer.get_id(),
            _ => false,
        }
    }

    fn query_slice(payload: &WorkPayload) -> &str {
        match payload {
            WorkPayload::Query(q) => q.as_str(),
            WorkPayload::Process(p) => p.as_str(),
            WorkPayload::Delegate { capability, .. } => capability.as_str(),
            WorkPayload::Research { .. } => "research",
            WorkPayload::Write { .. } => "write",
            WorkPayload::ScrubPII { .. } => "pii",
            WorkPayload::Review { .. } => "review",
            WorkPayload::CreateDocument { .. } => "document",
            WorkPayload::ImproveDocument { .. } => "improve-doc",
        }
    }

    fn handle_delegation(
        &mut self,
        envelope: &WorkEnvelope,
        target_capability: String,
        myself: &ActorRef<ExpertMsg>,
    ) -> Result<(), ExpertError> {
        let peer = self
            .peers
            .as_ref()
            .and_then(|m| m.get(&target_capability))
            .cloned()
            .ok_or_else(|| ExpertError::CapabilityNotFound(target_capability.clone()))?;

        let sub = match &envelope.payload {
            WorkPayload::Query(q) => q.clone(),
            WorkPayload::Process(p) => p.clone(),
            WorkPayload::Delegate { payload, .. } => payload.clone(),
            WorkPayload::Research { .. } => "research".to_string(),
            WorkPayload::Write { .. } => "write".to_string(),
            WorkPayload::ScrubPII { .. } => "pii".to_string(),
            WorkPayload::Review { .. } => "review".to_string(),
            WorkPayload::CreateDocument { .. } => "document".to_string(),
            WorkPayload::ImproveDocument { .. } => "improve-doc".to_string(),
        };

        let delegated =
            BatonPass::prepare_delegation(envelope, myself.clone(), WorkPayload::Query(sub));

        let pending_task = BatonPass::create_pending_task_with_peer(
            envelope,
            target_capability.clone(),
            peer.clone(),
        );
        self.base.store_pending_task(pending_task);

        tracing::info!(
            trace_id = %envelope.trace_id,
            from = %self.base.name,
            to = %target_capability,
            "Delegating to peer capability"
        );

        let _ = peer.cast(ExpertMsg::Work(delegated));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Actor for RustExpert {
    type Msg = ExpertMsg;
    type State = RustExpert;
    type Arguments = (
        Option<PeerMap>,
        Option<broadcast::Sender<crate::AgentEvent>>,
    );

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let (peers, mesh_events) = args;
        let mut s = Self::new();
        s.peers = peers;
        s.mesh_events = mesh_events;
        Ok(s)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
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
                    send_work_output(&envelope, format!("Error: {}", e));
                    return Ok(());
                }

                if let Some(delegation_target) = state.should_delegate(
                    Self::query_slice(&envelope.payload),
                    envelope.hop_count,
                    &envelope,
                ) {
                    let target_cap = delegation_target.clone();
                    match state.handle_delegation(&envelope, delegation_target, &myself) {
                        Ok(()) => {
                            crate::mesh::live::emit_mesh(
                                &state.mesh_events,
                                &state.base.name,
                                "delegatie",
                                &format!("→ {}", target_cap),
                            );
                        }
                        Err(e) => {
                            crate::mesh::live::emit_mesh(
                                &state.mesh_events,
                                &state.base.name,
                                "delegatie_fout",
                                &format!("{e}"),
                            );
                            send_work_output(&envelope, format!("Delegation mislukt: {}", e));
                        }
                    }
                    return Ok(());
                }

                let q_slice = Self::query_slice(&envelope.payload);
                let mut replied = false;
                if crate::mesh::ollama_bridge::mesh_ollama_enabled() {
                    match chat_via_ollama_for_mesh_with_events(
                        &state.base.name,
                        q_slice,
                        None,
                        &state.mesh_events,
                    )
                    .await
                    {
                        Ok(text) => {
                            send_work_output(
                                &envelope,
                                format!("[{} · Ollama] {}", state.base.name, text),
                            );
                            replied = true;
                        }
                        Err(e) => {
                            crate::mesh::live::emit_mesh(
                                &state.mesh_events,
                                &state.base.name,
                                "ollama_fallback",
                                &format!("{e}"),
                            );
                            tracing::warn!(
                                target: "mesh_ollama",
                                expert = %state.base.name,
                                error = %e,
                                "Ollama niet beschikbaar; heuristiek"
                            );
                        }
                    }
                }

                if !replied {
                    if state.can_handle_locally(q_slice) {
                        crate::mesh::live::emit_mesh(
                            &state.mesh_events,
                            &state.base.name,
                            "heuristiek",
                            "lokaal antwoord",
                        );
                        let result = state.process_locally(&envelope);
                        send_work_output(&envelope, result);
                        tracing::info!(
                            trace_id = %envelope.trace_id,
                            expert = %state.base.name,
                            "Processed locally (heuristiek)"
                        );
                    } else {
                        crate::mesh::live::emit_mesh(
                            &state.mesh_events,
                            &state.base.name,
                            "fallback",
                            "geen direct mesh-antwoord",
                        );
                        let result = format!(
                            "{}: Query understood but no direct answer available",
                            state.base.name
                        );
                        send_work_output(&envelope, result);
                    }
                }
            }
            ExpertMsg::SetPeers(peers) => {
                state.peers = peers;
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.base.pending_tasks.remove(&trace_id) {
                    tracing::info!(
                        trace_id = %trace_id,
                        peer_result = %result,
                        "Peer response received, resuming processing"
                    );

                    let final_result = format!("{} (peer-assisted): {}", state.base.name, result);

                    if let Some(ref parent) = task.reply_to {
                        let _ = parent.cast(ExpertMsg::PeerResponse {
                            trace_id,
                            result: final_result,
                        });
                    } else if let Some(ref gw) = task.entry_reply {
                        let _ = gw.cast(EntryMsg::ExpertResponse {
                            trace_id,
                            result: final_result,
                        });
                    }
                }
            }
            ExpertMsg::MeshSignal(signal) => match signal {
                MeshSignal::Cancel => {
                    tracing::info!(
                        expert = %state.base.name,
                        pending = state.base.pending_tasks.len(),
                        "Cancel signal received (poison pill)"
                    );
                    state.base.pending_tasks.clear();
                }
            },
            ExpertMsg::TimerTick => {
                let recovered = state.base.recover_expired_tasks(&myself);
                let purged = state.base.purge_expired_tasks();

                if !recovered.is_empty() || !purged.is_empty() {
                    tracing::debug!(
                        expert = %state.base.name,
                        recovered = recovered.len(),
                        purged = purged.len(),
                        remaining = state.base.pending_tasks.len(),
                        "Timer tick: stability maintenance"
                    );
                }
            }
        }
        Ok(())
    }
}

pub async fn spawn_rust_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, RustExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_rust_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, RustExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_rust_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) =
        Actor::spawn(None, RustExpert::with_timeout(timeout_secs), (None, None)).await?;
    Ok(actor_ref)
}
