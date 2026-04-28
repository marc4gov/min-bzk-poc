use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::types::{EntryMsg, MeshSignal};
use crate::mesh::expert::{
    send_work_output, BatonPass, ExpertError, ExpertMsg, ExpertState, PeerMap, WorkEnvelope,
    WorkPayload, StyleProfile,
};

const SCHRIJVER_CAPABILITIES: &[&str] = &["write", "draft", "compose"];

pub struct SchrijverExpert {
    base: ExpertState,
    style_templates: HashMap<StyleProfile, String>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl SchrijverExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "SchrijverExpert".to_string(),
            SCHRIJVER_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut style_templates = HashMap::new();
        style_templates.insert(
            StyleProfile::Formal,
            "Write in a professional, formal tone with proper structure and complete sentences.".to_string(),
        );
        style_templates.insert(
            StyleProfile::Casual,
            "Write in a friendly, conversational tone that is easy to read.".to_string(),
        );
        style_templates.insert(
            StyleProfile::Legal,
            "Write with legal precision, using appropriate terminology and disclaimers.".to_string(),
        );
        style_templates.insert(
            StyleProfile::Technical,
            "Write with technical accuracy, including relevant details and specifications.".to_string(),
        );

        Self {
            base,
            style_templates,
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

    fn generate_document(&mut self, notes: &str, style: &StyleProfile) -> String {
        self.processed_count += 1;

        let style_guidance = self
            .style_templates
            .get(style)
            .map(|s| s.as_str())
            .unwrap_or("Write clearly and concisely.");

        format!(
            "=== Document #{} ({:?}) ===\n\nStyle Guidance: {}\n\nResearch Notes:\n{}\n\n=== End Document ===",
            self.processed_count,
            style,
            style_guidance,
            notes
        )
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::Write {
                research_notes,
                style_profile,
            } => {
                let document = self.generate_document(research_notes, style_profile);
                tracing::info!(
                    style = ?style_profile,
                    notes_len = research_notes.len(),
                    "Generated document"
                );
                document
            }
            _ => "SchrijverExpert: Please provide Write payload with research_notes and style_profile".to_string(),
        }
    }

    fn should_delegate(&self, query: &str, hop_count: u32, _envelope: &WorkEnvelope) -> Option<String> {
        if hop_count >= 3 {
            return None;
        }

        let query_lower = query.to_lowercase();

        if query_lower.contains("research") || query_lower.contains("scrape") {
            Some("research".to_string())
        } else if query_lower.contains("pii") || query_lower.contains("anonymize") {
            Some("pii".to_string())
        } else {
            None
        }
    }

    fn would_delegate_to_sender(
        peers: &Option<PeerMap>,
        target: &str,
        envelope: &WorkEnvelope,
    ) -> bool {
        match (peers.as_ref().and_then(|m| m.get(target)), envelope.reply_to.as_ref()) {
            (Some(peer), Some(reply)) => reply.get_id() == peer.get_id(),
            _ => false,
        }
    }

    fn query_slice(payload: &WorkPayload) -> &str {
        match payload {
            WorkPayload::Query(q) => q.as_str(),
            WorkPayload::Process(p) => p.as_str(),
            WorkPayload::Delegate { capability, .. } => capability.as_str(),
            WorkPayload::Write { .. } => "write",
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

        let delegated = BatonPass::prepare_delegation(
            envelope,
            myself.clone(),
            envelope.payload.clone(),
        );

        let pending_task =
            BatonPass::create_pending_task_with_peer(envelope, target_capability.clone(), peer.clone());
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
impl Actor for SchrijverExpert {
    type Msg = ExpertMsg;
    type State = SchrijverExpert;
    type Arguments = (Option<PeerMap>, Option<broadcast::Sender<crate::AgentEvent>>);

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
                    let _ = state.handle_delegation(&envelope, delegation_target, &myself);
                } else {
                    let result = state.process_locally(&envelope);
                    send_work_output(&envelope, result);
                }
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.base.pending_tasks.remove(&trace_id) {
                    if let Some(ref reply_to) = task.reply_to {
                        let _ = reply_to.cast(ExpertMsg::PeerResponse {
                            trace_id,
                            result,
                        });
                    } else if let Some(ref gateway) = task.entry_reply {
                        let _ = gateway.cast(EntryMsg::ExpertResponse {
                            trace_id,
                            result,
                        });
                    }
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

pub async fn spawn_schrijver_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, SchrijverExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_schrijver_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, SchrijverExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_schrijver_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, SchrijverExpert::with_timeout(timeout_secs), (None, None)).await?;
    Ok(actor_ref)
}
