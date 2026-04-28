use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::types::{EntryMsg, MeshSignal};
use crate::mesh::expert::{
    send_work_output, BatonPass, ExpertMsg, ExpertState, PeerMap, WorkEnvelope, WorkPayload, PIICategory,
};

const PII_CAPABILITIES: &[&str] = &["pii", "anonymize", "scrub"];

#[derive(Debug, Clone)]
pub struct PIIResult {
    pub scrubbed_content: String,
    pub report: HashMap<String, Vec<String>>,
}

pub struct PIIStripperExpert {
    base: ExpertState,
    patterns: HashMap<PIICategory, regex::Regex>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl PIIStripperExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "PIIStripperExpert".to_string(),
            PII_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut patterns = HashMap::new();

        patterns.insert(PIICategory::Email,
            regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap());
        patterns.insert(PIICategory::PhoneNumber,
            regex::Regex::new(r"\+?\d{1,3}[-.\s]?\(?\d{1,4}\)?[-.\s]?\d{1,4}[-.\s]?\d{1,9}").unwrap());
        patterns.insert(PIICategory::SSN,
            regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b|\b\d{3}\s\d{2}\s\d{4}\b").unwrap());
        patterns.insert(PIICategory::IBAN,
            regex::Regex::new(r"[A-Z]{2}\d{2}[A-Z0-9]{11,30}").unwrap());
        patterns.insert(PIICategory::Name,
            regex::Regex::new(r"\b[A-Z][a-z]+\s[A-Z][a-z]+\b").unwrap());
        patterns.insert(PIICategory::Address,
            regex::Regex::new(r"\d+\s+[A-Z][a-z]+\s+(Street|St|Avenue|Ave|Road|Rd|Lane|Ln|Boulevard|Blvd)\b").unwrap());

        Self {
            base,
            patterns,
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

    fn scrub_pii(&mut self, content: &str, categories: &[PIICategory]) -> PIIResult {
        self.processed_count += 1;

        let mut scrubbed_content = content.to_string();
        let mut report: HashMap<String, Vec<String>> = HashMap::new();

        for category in categories {
            let category_name = format!("{:?}", category);

            if let Some(pattern) = self.patterns.get(category) {
                let matches: Vec<String> = pattern
                    .find_iter(content)
                    .map(|m| m.as_str().to_string())
                    .collect();

                if !matches.is_empty() {
                    report.insert(category_name.clone(), matches.clone());

                    scrubbed_content = pattern
                        .replace_all(&scrubbed_content, "[REDACTED]")
                        .to_string();
                }
            }
        }

        PIIResult {
            scrubbed_content,
            report,
        }
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::ScrubPII { content, pii_categories } => {
                let result = self.scrub_pii(content, pii_categories);

                let mut response = format!(
                    "PII Stripper Report #{}:\n",
                    self.processed_count
                );

                for (category, matches) in &result.report {
                    response.push_str(&format!(
                        "- {}: {} instance(s) found and redacted\n",
                        category,
                        matches.len()
                    ));
                }

                response.push_str(&format!("\nScrubbed content:\n{}\n", result.scrubbed_content));

                response
            }
            _ => "PIIStripperExpert: Please provide ScrubPII payload with content and categories".to_string(),
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
            WorkPayload::ScrubPII { .. } => "pii",
            _ => "",
        }
    }
}

#[async_trait::async_trait]
impl Actor for PIIStripperExpert {
    type Msg = ExpertMsg;
    type State = PIIStripperExpert;
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

                let result = state.process_locally(&envelope);
                send_work_output(&envelope, result);
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.base.pending_tasks.remove(&trace_id) {
                    if let Some(ref gateway) = task.entry_reply {
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

pub async fn spawn_pii_stripper_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, PIIStripperExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_pii_stripper_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, PIIStripperExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_pii_stripper_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, PIIStripperExpert::with_timeout(timeout_secs), (None, None)).await?;
    Ok(actor_ref)
}
