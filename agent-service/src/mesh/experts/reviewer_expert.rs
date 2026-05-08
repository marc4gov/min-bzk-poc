//! **ReviewerExpert** — heuristische regels (regellengte, TODO, criteria). Geschikt voor preflight checks;
//! diepgaande inhoudelijke review hoort bij een LM of menselijke QA.
//! Retourneert **de volledige tekst** met review-aanwijzingen in dezelfde output (geen los rapport zonder body).
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::expert::{
    deliver_peer_response_result, send_work_output, BatonPass, ExpertMsg, ExpertState, PeerMap,
    ReviewCriteria, WorkEnvelope, WorkPayload,
};
use crate::mesh::live::emit_mesh;
use crate::mesh::types::MeshSignal;

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
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl ReviewerExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "ReviewerExpert".to_string(),
            REVIEWER_CAPABILITIES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );

        Self {
            base,
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

    fn review_document(
        &mut self,
        content: &str,
        criteria: &Option<ReviewCriteria>,
    ) -> Vec<ReviewAnnotation> {
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
                        message: format!(
                            "Word count {} outside target range ({}, {})",
                            word_count, min, max
                        ),
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

    /// Volledige brontekst terug, met per regel (en globaal) wat er moet gebeuren.
    fn format_document_with_inline_review(
        &self,
        content: &str,
        annotations: &[ReviewAnnotation],
    ) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut by_line: HashMap<usize, Vec<&ReviewAnnotation>> = HashMap::new();
        let mut globals: Vec<&ReviewAnnotation> = Vec::new();

        for a in annotations {
            if a.line_number == 0 {
                globals.push(a);
            } else {
                by_line.entry(a.line_number).or_default().push(a);
            }
        }

        let mut out = String::new();
        out.push_str("=== Document inclusief review ===\n\n");

        if annotations.is_empty() {
            out.push_str("**Status:** geen automatische bevindingen op regelniveau.\n\n");
            out.push_str("---\n\n");
            out.push_str(content);
            out.push_str("\n\n=== Einde ===\n");
            return out;
        }

        out.push_str("**Status:** er zijn aanwijzingen onder de betreffende regels.\n\n");

        if !globals.is_empty() {
            out.push_str("### Algemene review-opmerkingen\n\n");
            for g in &globals {
                out.push_str(&format!(
                    "- **[{}]** {}",
                    g.severity.to_uppercase(),
                    g.message
                ));
                if let Some(s) = &g.suggestion {
                    out.push_str(&format!("\n  - *Actie:* {}", s));
                }
                out.push_str("\n");
            }
            out.push_str("\n---\n\n");
        }

        out.push_str("### Tekst met review-aanwijzingen\n\n");

        if lines.is_empty() && content.is_empty() {
            out.push_str("(lege invoer)\n");
        }

        for (i, line) in lines.iter().enumerate() {
            let line_no = i + 1;
            out.push_str(line);
            out.push('\n');
            if let Some(anns) = by_line.get(&line_no) {
                for a in anns {
                    out.push_str("  ▸ **[");
                    out.push_str(&a.severity.to_uppercase());
                    out.push_str("]** ");
                    out.push_str(&a.message);
                    if let Some(s) = &a.suggestion {
                        out.push_str(&format!(" — *te doen:* {}", s));
                    }
                    out.push('\n');
                }
            }
        }

        out.push_str("\n=== Einde ===\n");
        out
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::Review { content, criteria } => {
                let annotations = self.review_document(content, criteria);
                let report = self.format_document_with_inline_review(content, &annotations);

                tracing::info!(
                    issues_found = annotations.len(),
                    "Document review completed"
                );

                report
            }
            _ => "ReviewerExpert: Please provide Review payload with content".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Actor for ReviewerExpert {
    type Msg = ExpertMsg;
    type State = ReviewerExpert;
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

                let result = state.process_locally(&envelope);
                emit_mesh(
                    &state.mesh_events,
                    &state.base.name,
                    "review_klaar",
                    "Review uitgevoerd",
                );
                send_work_output(&envelope, result);
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
    let (actor_ref, _) =
        Actor::spawn(None, ReviewerExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_reviewer_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(
        None,
        ReviewerExpert::with_timeout(timeout_secs),
        (None, None),
    )
    .await?;
    Ok(actor_ref)
}
