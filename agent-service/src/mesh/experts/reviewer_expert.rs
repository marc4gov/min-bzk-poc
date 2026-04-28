use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::types::{EntryMsg, MeshSignal};
use crate::mesh::expert::{
    send_work_output, BatonPass, ExpertMsg, ExpertState, PeerMap, WorkEnvelope, WorkPayload,
    ReviewCriteria,
};

const REVIEWER_CAPABILITIES: &[&str] = &["review", "critique", "edit"];

#[derive(Debug, Clone)]
pub struct ReviewAnnotation {
    pub line_number: usize,
    pub severity: String,
    pub message: String,
    pub suggestion: Option<String>,
}

pub struct ReviewerExpert {
    base: ExpertState,
    review_templates: HashMap<String, Vec<String>>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl ReviewerExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "ReviewerExpert".to_string(),
            REVIEWER_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut review_templates = HashMap::new();
        review_templates.insert(
            "clarity".to_string(),
            vec![
                "Consider rewriting for clarity".to_string(),
                "This sentence could be more concise".to_string(),
            ],
        );
        review_templates.insert(
            "grammar".to_string(),
            vec!["Check grammar and punctuation".to_string()],
        );
        review_templates.insert(
            "structure".to_string(),
            vec![
                "Consider restructuring this paragraph".to_string(),
                "Add headings for better organization".to_string(),
            ],
        );

        Self {
            base,
            review_templates,
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

    fn review_document(&mut self, content: &str, criteria: &Option<ReviewCriteria>) -> Vec<ReviewAnnotation> {
        self.processed_count += 1;

        let mut annotations = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.len() > 150 {
                annotations.push(ReviewAnnotation {
                    line_number: i + 1,
                    severity: "warning".to_string(),
                    message: format!("Line exceeds recommended length ({} chars)", line.len()),
                    suggestion: Some("Consider splitting this line".to_string()),
                });
            }

            if line.contains("TODO") || line.contains("FIXME") {
                annotations.push(ReviewAnnotation {
                    line_number: i + 1,
                    severity: "info".to_string(),
                    message: "Contains TODO/FIXME marker".to_string(),
                    suggestion: Some("Resolve or track this item".to_string()),
                });
            }
        }

        if let Some(crit) = criteria {
            if let Some(ref tone) = crit.tone {
                annotations.push(ReviewAnnotation {
                    line_number: 0,
                    severity: "info".to_string(),
                    message: format!("Tone check: document should be '{}'", tone),
                    suggestion: None,
                });
            }

            if let Some((min, max)) = crit.length_constraints {
                let word_count = content.split_whitespace().count();
                if word_count < min || word_count > max {
                    annotations.push(ReviewAnnotation {
                        line_number: 0,
                        severity: "warning".to_string(),
                        message: format!("Word count {} outside target range ({}, {})", word_count, min, max),
                        suggestion: Some("Adjust content length".to_string()),
                    });
                }
            }

            for area in &crit.focus_areas {
                annotations.push(ReviewAnnotation {
                    line_number: 0,
                    severity: "info".to_string(),
                    message: format!("Focus area: {}", area),
                    suggestion: None,
                });
            }
        }

        annotations
    }

    fn format_review(&self, annotations: &[ReviewAnnotation]) -> String {
        if annotations.is_empty() {
            return "=== Review Report ===\n\nStatus: PASSED\n\nNo issues found. Document looks good!".to_string();
        }

        let mut report = format!(
            "=== Review Report #{} ===\n\nStatus: NEEDS ATTENTION\n\n",
            self.processed_count
        );

        report.push_str(&format!("Found {} issue(s):\n\n", annotations.len()));

        for (i, annotation) in annotations.iter().enumerate() {
            report.push_str(&format!(
                "{}. [{}] Line {}: {}\n",
                i + 1,
                annotation.severity.to_uppercase(),
                annotation.line_number,
                annotation.message
            ));

            if let Some(ref suggestion) = annotation.suggestion {
                report.push_str(&format!("   Suggestion: {}\n", suggestion));
            }

            report.push('\n');
        }

        report.push_str("=== End Review ===");

        report
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::Review { content, criteria } => {
                let annotations = self.review_document(content, criteria);
                let report = self.format_review(&annotations);

                tracing::info!(
                    issues_found = annotations.len(),
                    "Document review completed"
                );

                report
            }
            _ => "ReviewerExpert: Please provide Review payload with content".to_string(),
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
            WorkPayload::Review { .. } => "review",
            _ => "",
        }
    }
}

#[async_trait::async_trait]
impl Actor for ReviewerExpert {
    type Msg = ExpertMsg;
    type State = ReviewerExpert;
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

pub async fn spawn_reviewer_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ReviewerExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_reviewer_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ReviewerExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_reviewer_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ReviewerExpert::with_timeout(timeout_secs), (None, None)).await?;
    Ok(actor_ref)
}
