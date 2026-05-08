//! **ResearchExpert** — Stub (+ URL-gecorrigeerde preview als **`MESH_RESEARCH_HTTP=1`**/`true`/`yes`).
//! Bij fout tijdens GET wordt automatisch naar stub-gecombineerde teksten teruggevallen.

use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::expert::{
    deliver_peer_response_result, send_work_output, BatonPass, ExpertError, ExpertMsg, ExpertState,
    PeerMap, WorkEnvelope, WorkPayload, MAX_DELEGATION_DEPTH,
};
use crate::mesh::types::MeshSignal;
use crate::mesh::{live::emit_mesh, research_http};

const RESEARCH_CAPABILITIES: &[&str] = &["research", "scrape", "gather"];

pub struct ResearchExpert {
    base: ExpertState,
    domain_knowledge: HashMap<String, String>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl ResearchExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "ResearchExpert".to_string(),
            RESEARCH_CAPABILITIES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );

        let mut domain_knowledge = HashMap::new();
        domain_knowledge.insert(
            "scrape".to_string(),
            "URL scraping extracts content from web pages".to_string(),
        );
        domain_knowledge.insert(
            "gather".to_string(),
            "Information gathering from multiple sources".to_string(),
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
        query_lower.contains("http")
            || query_lower.contains("url")
            || query_lower.contains("scrape")
            || query_lower.contains("research")
    }

    fn research_stub_notes(&mut self, urls: &[String], depth: &u8) -> String {
        self.processed_count += 1;
        tracing::info!(
            urls = ?urls,
            depth = depth,
            "research stub (geen HTTP of fetch-fallback)"
        );

        let mut notes = format!(
            "Research from {} source(s) — stub (geen HTTP).\nTIP: export {}=1 voor GET-preview.\n\n",
            urls.len(),
            research_http::ENV_RESEARCH_HTTP
        );

        for (i, url) in urls.iter().enumerate() {
            notes.push_str(&format!("{}. bron: {}\n", i + 1, url));
            notes.push_str(&format!("   (stub: inhoud wordt hier niet geladen)\n"));
            notes.push_str(&format!("   diepte-parameter: {}\n\n", depth));
        }

        notes
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::Research { urls, depth } => {
                self.research_stub_notes(urls.as_slice(), depth)
            }
            _ => "ResearchExpert: Please provide Research payload with URLs".to_string(),
        }
    }

    async fn dispatch_research_reply(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::Research { urls, depth } => {
                if research_http::http_fetch_enabled() {
                    match research_http::fetch_research_notes(urls.as_slice(), *depth).await {
                        Ok(s) => return s,
                        Err(e) => {
                            tracing::warn!(error = %e, "HTTP research mislukt; gebruik stub");
                            emit_mesh(
                                &self.mesh_events,
                                &self.base.name,
                                "research_http_fallback",
                                &format!("fallback na: {e}"),
                            );
                        }
                    }
                }
                self.research_stub_notes(urls.as_slice(), depth)
            }
            _ => self.process_locally(envelope),
        }
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

        let target = if query_lower.contains("write") || query_lower.contains("draft") {
            Some("write".to_string())
        } else {
            None
        }?;

        if Self::would_delegate_to_sender(&self.peers, &target, envelope) {
            return None;
        }

        Some(target)
    }

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
            _ => "",
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

        let delegated =
            BatonPass::prepare_delegation(envelope, myself.clone(), envelope.payload.clone());

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
            "Delegating to peer"
        );

        let _ = peer.cast(ExpertMsg::Work(delegated));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Actor for ResearchExpert {
    type Msg = ExpertMsg;
    type State = ResearchExpert;
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
            ExpertMsg::SetPeers(peers) => {
                state.peers = peers;
                Ok(())
            }
            ExpertMsg::Work(envelope) => {
                state.base.cleanup_expired_tasks();

                if let Err(e) = BatonPass::check_hop_limit(envelope.hop_count) {
                    send_work_output(&envelope, format!("Error: {}", e));
                    return Ok(());
                }

                if let Some(delegation_target) = state.should_delegate(
                    Self::query_slice(&envelope.payload),
                    envelope.hop_count,
                    &envelope,
                ) {
                    let cap = delegation_target.clone();
                    match state.handle_delegation(&envelope, delegation_target, &myself) {
                        Ok(()) => emit_mesh(
                            &state.mesh_events,
                            &state.base.name,
                            "delegatie",
                            &format!("→ {}", cap),
                        ),
                        Err(e) => {
                            emit_mesh(
                                &state.mesh_events,
                                &state.base.name,
                                "delegatie_fout",
                                &format!("{e}"),
                            );
                            send_work_output(&envelope, format!("Delegatie mislukt: {}", e));
                        }
                    }
                } else if state.can_handle_locally(Self::query_slice(&envelope.payload)) {
                    let result = state.dispatch_research_reply(&envelope).await;
                    emit_mesh(
                        &state.mesh_events,
                        &state.base.name,
                        "research_klaar",
                        "notities terug naar orchestrator",
                    );
                    send_work_output(&envelope, result);
                } else {
                    send_work_output(
                        &envelope,
                        format!("{}: Cannot handle this request type", state.base.name),
                    );
                }
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.base.pending_tasks.remove(&trace_id) {
                    deliver_peer_response_result(&task, trace_id, result);
                }
                Ok(())
            }
            ExpertMsg::MeshSignal(signal) => match signal {
                MeshSignal::Cancel => {
                    state.base.pending_tasks.clear();
                    Ok(())
                }
            },
            ExpertMsg::TimerTick => {
                let _recovered = state.base.recover_expired_tasks(&myself);
                let _purged = state.base.purge_expired_tasks();
                Ok(())
            }
        }
    }
}

pub async fn spawn_research_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ResearchExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_research_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) =
        Actor::spawn(None, ResearchExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_research_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(
        None,
        ResearchExpert::with_timeout(timeout_secs),
        (None, None),
    )
    .await?;
    Ok(actor_ref)
}
