use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use uuid::Uuid;

use crate::mesh::expert::{
    ErrorStrategy, ExpertMsg, PeerMap, ReviewCriteria, StyleProfile,
    WorkflowStep, WorkflowState, WorkEnvelope, WorkPayload, PIICategory,
};
use crate::mesh::types::{EntryMsg, MeshSignal, SessionContext};

const ORCHESTRATOR_CAPABILITIES: &[&str] = &["document", "create-doc"];

pub struct DocumentOrchestrator {
    name: String,
    pending_workflows: HashMap<Uuid, WorkflowState>,
    peers: Option<PeerMap>,
    timeout_secs: u64,
}

impl DocumentOrchestrator {
    pub fn new() -> Self {
        Self {
            name: "DocumentOrchestrator".to_string(),
            pending_workflows: HashMap::new(),
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

    fn start_workflow(&mut self, msg: CreateDocumentMsg) -> Result<(), String> {
        let mut state = WorkflowState {
            trace_id: msg.trace_id,
            current_step: WorkflowStep::Research,
            accumulated_results: HashMap::new(),
            error_strategy: msg.error_strategy.clone(),
            retry_count: 0,
            reply_to: msg.reply_to.clone(),
        };

        self.pending_workflows.insert(msg.trace_id, state.clone());

        let research_envelope = WorkEnvelope {
            payload: WorkPayload::Research {
                urls: msg.urls.clone(),
                depth: 2,
            },
            context: SessionContext {
                session_id: Uuid::new_v4(),
                user_id: "orchestrator".to_string(),
                metadata: HashMap::new(),
            },
            reply_to: None,
            entry_reply: msg.reply_to.clone(),
            trace_id: msg.trace_id,
            hop_count: 0,
        };

        if let Some(peer) = self.get_peer("research") {
            let _ = peer.cast(ExpertMsg::Work(research_envelope));
            Ok(())
        } else {
            Err("ResearchExpert not found".to_string())
        }
    }

    fn advance_workflow(&mut self, trace_id: Uuid, step: WorkflowStep, result: String) {
        if let Some(state) = self.pending_workflows.get_mut(&trace_id) {
            state.accumulated_results.insert(step.clone(), result);

            let next_step = match step {
                WorkflowStep::Research => WorkflowStep::Write,
                WorkflowStep::Write => WorkflowStep::ScrubPII,
                WorkflowStep::ScrubPII => WorkflowStep::Review,
                WorkflowStep::Review => {
                    self.finalize_workflow(trace_id);
                    return;
                }
            };

            state.current_step = next_step.clone();

            let envelope = match next_step {
                WorkflowStep::Write => {
                    let notes = state.accumulated_results.get(&WorkflowStep::Research)
                        .cloned()
                        .unwrap_or_default();

                    WorkEnvelope {
                        payload: WorkPayload::Write {
                            research_notes: notes,
                            style_profile: StyleProfile::Formal,
                        },
                        context: SessionContext {
                            session_id: Uuid::new_v4(),
                            user_id: "orchestrator".to_string(),
                            metadata: HashMap::new(),
                        },
                        reply_to: None,
                        entry_reply: state.reply_to.clone(),
                        trace_id,
                        hop_count: 0,
                    }
                }
                WorkflowStep::ScrubPII => {
                    let draft = state.accumulated_results.get(&WorkflowStep::Write)
                        .cloned()
                        .unwrap_or_default();

                    WorkEnvelope {
                        payload: WorkPayload::ScrubPII {
                            content: draft,
                            pii_categories: vec![PIICategory::Email, PIICategory::Name],
                        },
                        context: SessionContext {
                            session_id: Uuid::new_v4(),
                            user_id: "orchestrator".to_string(),
                            metadata: HashMap::new(),
                        },
                        reply_to: None,
                        entry_reply: state.reply_to.clone(),
                        trace_id,
                        hop_count: 0,
                    }
                }
                WorkflowStep::Review => {
                    let clean = state.accumulated_results.get(&WorkflowStep::ScrubPII)
                        .cloned()
                        .unwrap_or_default();

                    WorkEnvelope {
                        payload: WorkPayload::Review {
                            content: clean,
                            criteria: Some(ReviewCriteria {
                                tone: Some("professional".to_string()),
                                length_constraints: Some((100, 5000)),
                                focus_areas: vec!["clarity".to_string()],
                            }),
                        },
                        context: SessionContext {
                            session_id: Uuid::new_v4(),
                            user_id: "orchestrator".to_string(),
                            metadata: HashMap::new(),
                        },
                        reply_to: None,
                        entry_reply: state.reply_to.clone(),
                        trace_id,
                        hop_count: 0,
                    }
                }
                _ => return,
            };

            let capability = match next_step {
                WorkflowStep::Write => "write",
                WorkflowStep::ScrubPII => "pii",
                WorkflowStep::Review => "review",
                _ => return,
            };

            if let Some(peer) = self.get_peer(capability) {
                let _ = peer.cast(ExpertMsg::Work(envelope));
            } else {
                self.send_error(trace_id, &format!("{} not found", capability));
            }
        }
    }

    fn finalize_workflow(&mut self, trace_id: Uuid) {
        if let Some(state) = self.pending_workflows.remove(&trace_id) {
            let final_result = state
                .accumulated_results
                .get(&WorkflowStep::Review)
                .cloned()
                .unwrap_or_else(|| {
                    format!(
                        "Document created (steps: {:?})",
                        state.accumulated_results.keys().collect::<Vec<_>>()
                    )
                });

            if let Some(reply_to) = state.reply_to {
                let _ = reply_to.cast(EntryMsg::ExpertResponse {
                    trace_id,
                    result: final_result,
                });
            }
        }
    }

    fn send_error(&self, trace_id: Uuid, error: &str) {
        if let Some(state) = self.pending_workflows.get(&trace_id) {
            if let Some(reply_to) = &state.reply_to {
                let _ = reply_to.cast(EntryMsg::ExpertResponse {
                    trace_id,
                    result: format!("Error: {}", error),
                });
            }
        }
    }

    fn get_peer(&self, capability: &str) -> Option<ActorRef<ExpertMsg>> {
        self.peers.as_ref()?.get(capability).cloned()
    }
}

#[derive(Debug, Clone)]
struct CreateDocumentMsg {
    urls: Vec<String>,
    style_profile: StyleProfile,
    pii_categories: Vec<PIICategory>,
    review_criteria: Option<ReviewCriteria>,
    error_strategy: ErrorStrategy,
    reply_to: Option<ActorRef<EntryMsg>>,
    trace_id: Uuid,
}

#[async_trait::async_trait]
impl Actor for DocumentOrchestrator {
    type Msg = ExpertMsg;
    type State = DocumentOrchestrator;
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
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ExpertMsg::SetPeers(peers) => {
                state.peers = peers;
                Ok(())
            }
            ExpertMsg::Work(envelope) => {
                if let WorkPayload::CreateDocument {
                    urls,
                    style_profile,
                    pii_categories,
                    review_criteria,
                    error_strategy,
                } = envelope.payload.clone()
                {
                    let msg = CreateDocumentMsg {
                        urls,
                        style_profile,
                        pii_categories,
                        review_criteria,
                        error_strategy,
                        reply_to: envelope.entry_reply,
                        trace_id: envelope.trace_id,
                    };

                    if let Err(e) = state.start_workflow(msg) {
                        state.send_error(envelope.trace_id, &e);
                    }
                }
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                let step = state
                    .pending_workflows
                    .get(&trace_id)
                    .map(|s| s.current_step.clone());

                if let Some(step) = step {
                    state.advance_workflow(trace_id, step, result);
                }
                Ok(())
            }
            ExpertMsg::MeshSignal(MeshSignal::Cancel) => {
                state.pending_workflows.clear();
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

pub async fn spawn_document_orchestrator(
    peers: PeerMap,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, DocumentOrchestrator::new(), Some(peers)).await?;
    Ok(actor_ref)
}
