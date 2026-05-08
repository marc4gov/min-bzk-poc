use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use uuid::Uuid;

use crate::mesh::expert::{
    ErrorStrategy, ExpertMsg, PIICategory, PeerMap, WorkEnvelope, WorkPayload, WorkflowStep,
};
use crate::mesh::types::{EntryMsg, SessionContext};

const IMPROVER_CAPABILITIES: &[&str] = &["improve-doc", "document-upload"];

struct ActiveImprovement {
    trace_id: Uuid,
    content: String,
    filename: String,
    instructions: String,
    pii_categories: Vec<PIICategory>,
    current_step: ImprovementStep,
    accumulated: HashMap<ImprovementStep, String>,
    reply_to: Option<ActorRef<EntryMsg>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ImprovementStep {
    Analyze,
    Improve,
    ScrubPII,
}

pub struct DocumentImprover {
    name: &'static str,
    pending: HashMap<Uuid, ActiveImprovement>,
    peers: Option<PeerMap>,
    timeout_secs: u64,
}

impl DocumentImprover {
    pub fn new() -> Self {
        tracing::debug!(
            orchestrator = "DocumentImprover",
            capabilities = IMPROVER_CAPABILITIES.join(","),
            "initialized"
        );
        Self {
            name: "DocumentImprover",
            pending: HashMap::new(),
            peers: None,
            timeout_secs: 60,
        }
    }

    pub fn with_peers(mut self, peers: PeerMap) -> Self {
        self.peers = Some(peers);
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    fn start_improvement(
        &mut self,
        content: String,
        filename: String,
        instructions: String,
        pii_categories: Vec<PIICategory>,
        trace_id: Uuid,
        reply_to: Option<ActorRef<EntryMsg>>,
        orchestrator: ActorRef<ExpertMsg>,
    ) {
        let active = ActiveImprovement {
            trace_id,
            content: content.clone(),
            filename: filename.clone(),
            instructions,
            pii_categories,
            current_step: ImprovementStep::Analyze,
            accumulated: HashMap::new(),
            reply_to,
        };

        self.pending.insert(trace_id, active);

        // Start met analyse
        let envelope = WorkEnvelope {
            payload: WorkPayload::Process(format!(
                "ANALYSE:\nBestand: {}\n\nInhoud:\n{}\n\nAnalyseer de structuur, hoofdlijnen en potentiële verbeterpunten.",
                filename, content
            )),
            context: SessionContext {
                session_id: Uuid::new_v4(),
                user_id: "improver".to_string(),
                metadata: HashMap::new(),
            },
            reply_to: Some(orchestrator.clone()),
            entry_reply: None,
            trace_id,
            hop_count: 0,
        };

        if let Some(peer) = self.get_peer("research") {
            let _ = peer.cast(ExpertMsg::Work(envelope));
            tracing::trace!(%trace_id, "document analysis started");
        } else {
            // Fallback: ga direct naar improve stap
            self.advance_to_improve(trace_id, orchestrator);
        }
    }

    fn advance_to_improve(&mut self, trace_id: Uuid, orchestrator: ActorRef<ExpertMsg>) {
        if let Some(active) = self.pending.get_mut(&trace_id) {
            active.current_step = ImprovementStep::Improve;

            let analysis = active
                .accumulated
                .get(&ImprovementStep::Analyze)
                .cloned()
                .unwrap_or_else(|| "(Geen analyse beschikbaar)".to_string());

            let prompt = format!(
                "VERBETER INSTRUCTIES: {}\n\nANALYSE:\n{}\n\nORIGINELE TEKST:\n{}\n\nVerbeter de tekst op basis van de instructies en analyse. Behoud de kernboodschap maar maak het duidelijker, professioneler en effectiever.",
                active.instructions, analysis, active.content
            );

            let envelope = WorkEnvelope {
                payload: WorkPayload::Write {
                    research_notes: prompt,
                    style_profile: crate::mesh::expert::StyleProfile::Custom("Professional improvement".to_string()),
                },
                context: SessionContext {
                    session_id: Uuid::new_v4(),
                    user_id: "improver".to_string(),
                    metadata: HashMap::new(),
                },
                reply_to: Some(orchestrator.clone()),
                entry_reply: None,
                trace_id,
                hop_count: 0,
            };

            if let Some(peer) = self.get_peer("write") {
                let _ = peer.cast(ExpertMsg::Work(envelope));
                tracing::trace!(%trace_id, "document improvement started");
            } else {
                self.send_error(trace_id, "Write expert niet gevonden");
            }
        }
    }

    fn advance_to_scrub(&mut self, trace_id: Uuid, orchestrator: ActorRef<ExpertMsg>) {
        if let Some(active) = self.pending.get_mut(&trace_id) {
            active.current_step = ImprovementStep::ScrubPII;

            let improved = active
                .accumulated
                .get(&ImprovementStep::Improve)
                .cloned()
                .unwrap_or_default();

            let categories = if active.pii_categories.is_empty() {
                vec![
                    PIICategory::Email,
                    PIICategory::PhoneNumber,
                    PIICategory::Name,
                    PIICategory::Address,
                ]
            } else {
                active.pii_categories.clone()
            };

            let envelope = WorkEnvelope {
                payload: WorkPayload::ScrubPII {
                    content: improved,
                    pii_categories: categories,
                },
                context: SessionContext {
                    session_id: Uuid::new_v4(),
                    user_id: "improver".to_string(),
                    metadata: HashMap::new(),
                },
                reply_to: Some(orchestrator.clone()),
                entry_reply: None,
                trace_id,
                hop_count: 0,
            };

            if let Some(peer) = self.get_peer("pii") {
                let _ = peer.cast(ExpertMsg::Work(envelope));
                tracing::trace!(%trace_id, "PII scrubbing started");
            } else {
                // Fallback: finalize zonder scrub
                self.finalize_improvement(trace_id);
            }
        }
    }

    fn finalize_improvement(&mut self, trace_id: Uuid) {
        if let Some(active) = self.pending.remove(&trace_id) {
            let final_result = active
                .accumulated
                .get(&ImprovementStep::ScrubPII)
                .cloned()
                .or_else(|| active.accumulated.get(&ImprovementStep::Improve).cloned())
                .unwrap_or_else(|| "Verbetering mislukt - geen resultaat".to_string());

            tracing::debug!(orchestrator = self.name, %trace_id, "improvement finalized");

            if let Some(reply_to) = active.reply_to {
                let _ = reply_to.cast(EntryMsg::ExpertResponse {
                    trace_id,
                    result: final_result,
                });
            }
        }
    }

    fn send_error(&self, trace_id: Uuid, error: &str) {
        if let Some(active) = self.pending.get(&trace_id) {
            tracing::warn!(orchestrator = self.name, %trace_id, error, "improvement error");
            if let Some(reply_to) = &active.reply_to {
                let _ = reply_to.cast(EntryMsg::ExpertResponse {
                    trace_id,
                    result: format!("Fout: {}", error),
                });
            }
        }
    }

    fn get_peer(&self, capability: &str) -> Option<ActorRef<ExpertMsg>> {
        self.peers.as_ref()?.get(capability).cloned()
    }
}

#[async_trait::async_trait]
impl Actor for DocumentImprover {
    type Msg = ExpertMsg;
    type State = DocumentImprover;
    type Arguments = Option<PeerMap>;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Option<PeerMap>,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::new().with_peers(args.unwrap_or_default()))
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
                if let WorkPayload::ImproveDocument {
                    content,
                    filename,
                    instructions,
                    pii_categories,
                } = envelope.payload
                {
                    state.start_improvement(
                        content,
                        filename,
                        instructions,
                        pii_categories,
                        envelope.trace_id,
                        envelope.entry_reply,
                        myself.clone(),
                    );
                }
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(active) = state.pending.get(&trace_id) {
                    let step = active.current_step.clone();

                    // Sla resultaat op
                    if let Some(a) = state.pending.get_mut(&trace_id) {
                        a.accumulated.insert(step.clone(), result);
                    }

                    // Ga naar volgende stap
                    match step {
                        ImprovementStep::Analyze => {
                            state.advance_to_improve(trace_id, myself.clone());
                        }
                        ImprovementStep::Improve => {
                            state.advance_to_scrub(trace_id, myself.clone());
                        }
                        ImprovementStep::ScrubPII => {
                            state.finalize_improvement(trace_id);
                        }
                    }
                }
                Ok(())
            }
            ExpertMsg::MeshSignal(crate::mesh::types::MeshSignal::Cancel) => {
                state.pending.clear();
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

pub async fn spawn_document_improver(
    peers: PeerMap,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, DocumentImprover::new(), Some(peers)).await?;
    Ok(actor_ref)
}
