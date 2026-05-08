use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use uuid::Uuid;

use crate::mesh::expert::{
    ErrorStrategy, ExpertMsg, PIICategory, PeerMap, ReviewCriteria, StyleProfile, WorkEnvelope,
    WorkPayload, WorkflowState, WorkflowStep,
};
use crate::mesh::types::{EntryMsg, MeshSignal, SessionContext};

const ORCHESTRATOR_CAPABILITIES: &[&str] = &["document", "create-doc"];

/// Alles wat uit `CreateDocument` komt om later stappen (Write / Scrub / Review) parametrisch uit te voeren.
struct ActiveWorkflow {
    inner: WorkflowState,
    style_profile: StyleProfile,
    pii_categories: Vec<PIICategory>,
    review_criteria: Option<ReviewCriteria>,
}

impl ActiveWorkflow {
    fn from_create(msg: CreateDocumentMsg) -> Self {
        Self {
            inner: WorkflowState {
                trace_id: msg.trace_id,
                current_step: WorkflowStep::Research,
                accumulated_results: HashMap::new(),
                error_strategy: msg.error_strategy.clone(),
                retry_count: 0,
                reply_to: msg.reply_to.clone(),
            },
            style_profile: msg.style_profile,
            pii_categories: msg.pii_categories,
            review_criteria: msg.review_criteria,
        }
    }
}

pub struct DocumentOrchestrator {
    name: &'static str,
    pending_workflows: HashMap<Uuid, ActiveWorkflow>,
    peers: Option<PeerMap>,
    timeout_secs: u64,
}

impl DocumentOrchestrator {
    pub fn new() -> Self {
        tracing::debug!(
            orchestrator = "DocumentOrchestrator",
            capabilities = ORCHESTRATOR_CAPABILITIES.join(","),
            "initialized"
        );
        Self {
            name: "DocumentOrchestrator",
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

    fn start_workflow(
        &mut self,
        msg: CreateDocumentMsg,
        orchestrator: ActorRef<ExpertMsg>,
    ) -> Result<(), String> {
        let trace_id = msg.trace_id;
        let urls = msg.urls.clone();
        let entry_reply = msg.reply_to.clone();

        let research_envelope = WorkEnvelope {
            payload: WorkPayload::Research { urls, depth: 2 },
            context: SessionContext {
                session_id: Uuid::new_v4(),
                user_id: "orchestrator".to_string(),
                metadata: HashMap::new(),
            },
            reply_to: Some(orchestrator),
            entry_reply,
            trace_id,
            hop_count: 0,
        };

        let wf = ActiveWorkflow::from_create(msg);
        self.pending_workflows.insert(trace_id, wf);

        if let Some(peer) = self.get_peer("research") {
            let _ = peer.cast(ExpertMsg::Work(research_envelope));
            tracing::trace!(%trace_id, orchestrator = self.name, "research phase started");
            Ok(())
        } else {
            self.pending_workflows.remove(&trace_id);
            Err("ResearchExpert not found".to_string())
        }
    }

    fn advance_workflow(
        &mut self,
        trace_id: Uuid,
        step: WorkflowStep,
        result: String,
        orchestrator: ActorRef<ExpertMsg>,
    ) {
        if let Some(active) = self.pending_workflows.get_mut(&trace_id) {
            active
                .inner
                .accumulated_results
                .insert(step.clone(), result);

            let next_step = match step {
                WorkflowStep::Research => WorkflowStep::Write,
                WorkflowStep::Write => WorkflowStep::ScrubPII,
                WorkflowStep::ScrubPII => WorkflowStep::Review,
                WorkflowStep::Review => {
                    self.finalize_workflow(trace_id);
                    return;
                }
            };

            active.inner.current_step = next_step.clone();

            let envelope = match next_step {
                WorkflowStep::Write => {
                    let notes = active
                        .inner
                        .accumulated_results
                        .get(&WorkflowStep::Research)
                        .cloned()
                        .unwrap_or_default();

                    WorkEnvelope {
                        payload: WorkPayload::Write {
                            research_notes: notes,
                            style_profile: active.style_profile.clone(),
                        },
                        context: SessionContext {
                            session_id: Uuid::new_v4(),
                            user_id: "orchestrator".to_string(),
                            metadata: HashMap::new(),
                        },
                        reply_to: Some(orchestrator.clone()),
                        entry_reply: active.inner.reply_to.clone(),
                        trace_id,
                        hop_count: 0,
                    }
                }
                WorkflowStep::ScrubPII => {
                    let draft = active
                        .inner
                        .accumulated_results
                        .get(&WorkflowStep::Write)
                        .cloned()
                        .unwrap_or_default();

                    let categories = if active.pii_categories.is_empty() {
                        vec![PIICategory::Email, PIICategory::Name]
                    } else {
                        active.pii_categories.clone()
                    };

                    WorkEnvelope {
                        payload: WorkPayload::ScrubPII {
                            content: draft,
                            pii_categories: categories,
                        },
                        context: SessionContext {
                            session_id: Uuid::new_v4(),
                            user_id: "orchestrator".to_string(),
                            metadata: HashMap::new(),
                        },
                        reply_to: Some(orchestrator.clone()),
                        entry_reply: active.inner.reply_to.clone(),
                        trace_id,
                        hop_count: 0,
                    }
                }
                WorkflowStep::Review => {
                    let clean = active
                        .inner
                        .accumulated_results
                        .get(&WorkflowStep::ScrubPII)
                        .cloned()
                        .unwrap_or_default();

                    let criteria = active.review_criteria.clone().or_else(|| {
                        Some(ReviewCriteria {
                            tone: Some("professional".to_string()),
                            length_constraints: Some((100, 5000)),
                            focus_areas: vec!["clarity".to_string()],
                        })
                    });

                    WorkEnvelope {
                        payload: WorkPayload::Review {
                            content: clean,
                            criteria,
                        },
                        context: SessionContext {
                            session_id: Uuid::new_v4(),
                            user_id: "orchestrator".to_string(),
                            metadata: HashMap::new(),
                        },
                        reply_to: Some(orchestrator.clone()),
                        entry_reply: active.inner.reply_to.clone(),
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
                tracing::trace!(%trace_id, orchestrator = self.name, step = ?next_step, "workflow advanced");
            } else {
                self.send_error(trace_id, &format!("{} not found", capability));
            }
        }
    }

    fn finalize_workflow(&mut self, trace_id: Uuid) {
        if let Some(active) = self.pending_workflows.remove(&trace_id) {
            let final_result = active
                .inner
                .accumulated_results
                .get(&WorkflowStep::Review)
                .cloned()
                .unwrap_or_else(|| {
                    format!(
                        "DocumentOrchestrator: geen Review-stap-resultaat (stappen: {:?})",
                        active.inner.accumulated_results.keys().collect::<Vec<_>>()
                    )
                });

            tracing::debug!(orchestrator = self.name, %trace_id, "workflow finalized");

            if let Some(reply_to) = active.inner.reply_to {
                let _ = reply_to.cast(EntryMsg::ExpertResponse {
                    trace_id,
                    result: final_result,
                });
            }
        }
    }

    fn send_error(&self, trace_id: Uuid, error: &str) {
        if let Some(active) = self.pending_workflows.get(&trace_id) {
            tracing::warn!(orchestrator = self.name, %trace_id, error, "workflow error");
            if let Some(reply_to) = &active.inner.reply_to {
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

                    if let Err(e) = state.start_workflow(msg, myself.clone()) {
                        state.send_error(envelope.trace_id, &e);
                    }
                }
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                let step = state
                    .pending_workflows
                    .get(&trace_id)
                    .map(|w| w.inner.current_step.clone());

                if let Some(step) = step {
                    state.advance_workflow(trace_id, step, result, myself.clone());
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
