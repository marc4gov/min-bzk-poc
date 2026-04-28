use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use uuid::Uuid;

use crate::mesh::types::{EntryMsg, MeshSignal, RegistryMsg};
use crate::mesh::expert::{ExpertMsg, WorkEnvelope, WorkPayload};

const MAX_PENDING_REQUESTS: usize = 100;
const DEFAULT_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub trace_id: Uuid,
    pub original_context: crate::mesh::types::SessionContext,
    pub reply_to: Option<ActorRef<EntryMsg>>,
    pub expert: Option<ActorRef<ExpertMsg>>,
    pub created_at: std::time::Instant,
    pub completed: bool,
}

impl PendingRequest {
    pub fn new(
        trace_id: Uuid,
        original_context: crate::mesh::types::SessionContext,
        reply_to: Option<ActorRef<EntryMsg>>,
    ) -> Self {
        Self {
            trace_id,
            original_context,
            reply_to,
            expert: None,
            created_at: std::time::Instant::now(),
            completed: false,
        }
    }

    pub fn is_expired(&self, timeout_secs: u64) -> bool {
        self.created_at.elapsed().as_secs() > timeout_secs
    }
}

#[derive(Clone)]
pub struct EntryActor {
    registry: Option<ActorRef<RegistryMsg>>,
    pending_requests: HashMap<Uuid, PendingRequest>,
    timeout_secs: u64,
    experts: HashMap<String, ActorRef<ExpertMsg>>,
}

impl EntryActor {
    pub fn new() -> Self {
        Self {
            registry: None,
            pending_requests: HashMap::new(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            experts: HashMap::new(),
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            registry: None,
            pending_requests: HashMap::new(),
            timeout_secs,
            experts: HashMap::new(),
        }
    }

    pub fn set_registry(&mut self, registry: ActorRef<RegistryMsg>) {
        self.registry = Some(registry);
    }

    pub fn register_expert(&mut self, capability: String, expert: ActorRef<ExpertMsg>) {
        self.experts.insert(capability, expert);
    }

    fn cleanup_expired_requests(&mut self) -> Vec<Uuid> {
        let expired: Vec<Uuid> = self
            .pending_requests
            .iter()
            .filter(|(_, req)| req.is_expired(self.timeout_secs))
            .map(|(id, _)| *id)
            .collect();

        for id in &expired {
            self.pending_requests.remove(id);
        }

        expired
    }

    fn triage_capability(&self, query: &str) -> String {
        let query_lower = query.to_lowercase();

        if query_lower.contains("rust")
            || query_lower.contains("memory")
            || query_lower.contains("thread")
            || query_lower.contains("wasm")
        {
            "rust".to_string()
        } else if query_lower.contains("frontend")
            || query_lower.contains("ui")
            || query_lower.contains("react")
        {
            "frontend".to_string()
        } else if query_lower.contains("database") || query_lower.contains("sql") {
            "database".to_string()
        } else if query_lower.contains("research") || query_lower.contains("scrape") || query_lower.contains("gather") {
            "research".to_string()
        } else if query_lower.contains("write") || query_lower.contains("draft") || query_lower.contains("compose") {
            "write".to_string()
        } else if query_lower.contains("pii") || query_lower.contains("anonymize") || query_lower.contains("scrub") {
            "pii".to_string()
        } else if query_lower.contains("review") || query_lower.contains("critique") || query_lower.contains("edit") {
            "review".to_string()
        } else if query_lower.contains("document") || query_lower.contains("create doc") {
            "document".to_string()
        } else {
            "general".to_string()
        }
    }

    fn find_expert_for_capability(&self, capability: &str) -> Option<ActorRef<ExpertMsg>> {
        self.experts.get(capability).cloned()
    }

    fn deliver_final_response(
        trace_id: Uuid,
        result: String,
        reply_to: Option<ActorRef<EntryMsg>>,
    ) {
        if let Some(reply_to_ref) = reply_to {
            let _ = reply_to_ref.cast(EntryMsg::ExpertResponse {
                trace_id,
                result,
            });
        }
    }

    fn cancel_downstream(trace_id: Uuid, state: &EntryActor) {
        if let Some(request) = state.pending_requests.get(&trace_id) {
            if let Some(expert) = &request.expert {
                let _ = expert.cast(ExpertMsg::MeshSignal(MeshSignal::Cancel));
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EntryError {
    #[error("Registry not configured")]
    RegistryNotConfigured,

    #[error("No expert found for capability: {0}")]
    ExpertNotFound(String),

    #[error("Too many pending requests")]
    TooManyPendingRequests,

    #[error("Request timeout: {0}")]
    RequestTimeout(Uuid),
}

#[async_trait::async_trait]
impl Actor for EntryActor {
    type Msg = EntryMsg;
    type State = EntryActor;
    type Arguments = Option<ActorRef<RegistryMsg>>;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Option<ActorRef<RegistryMsg>>,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut state = self.clone();
        if let Some(registry) = args {
            state.set_registry(registry);
        }
        Ok(state)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            EntryMsg::SubmitRequest { query, context, reply_to } => {
                state.cleanup_expired_requests();

                if state.pending_requests.len() >= MAX_PENDING_REQUESTS {
                    tracing::warn!("Too many pending requests, rejecting new request");
                    EntryActor::deliver_final_response(
                        Uuid::new_v4(),
                        "Error: Too many pending requests".to_string(),
                        reply_to,
                    );
                    return Ok(());
                }

                let trace_id = Uuid::new_v4();
                let capability = state.triage_capability(&query);

                tracing::info!(
                    trace_id = %trace_id,
                    capability = %capability,
                    query = %query,
                    "Triage request to capability"
                );

                let expert = state.find_expert_for_capability(&capability);

                if let Some(expert_ref) = expert {
                    let mut request = PendingRequest::new(trace_id, context.clone(), reply_to);
                    request.expert = Some(expert_ref.clone());

                    let work_envelope = WorkEnvelope {
                        payload: WorkPayload::Query(query.clone()),
                        context,
                        reply_to: None,
                        entry_reply: Some(myself.clone()),
                        trace_id,
                        hop_count: 0,
                    };

                    let _ = expert_ref.cast(ExpertMsg::Work(work_envelope));
                    state.pending_requests.insert(trace_id, request);
                } else {
                    tracing::warn!(
                        capability = %capability,
                        "No expert found for capability"
                    );
                    EntryActor::deliver_final_response(
                        trace_id,
                        format!("Error: No expert found for capability: {}", capability),
                        reply_to,
                    );
                }
            }
            EntryMsg::ExpertResponse { trace_id, result } => {
                if let Some(request) = state.pending_requests.remove(&trace_id) {
                    tracing::info!(
                        trace_id = %trace_id,
                        result_len = result.len(),
                        "Final expert response received"
                    );
                    EntryActor::deliver_final_response(trace_id, result, request.reply_to);
                } else {
                    tracing::warn!(
                        trace_id = %trace_id,
                        "Received response for unknown request"
                    );
                }
            }
            EntryMsg::Cancel { trace_id } => {
                if state.pending_requests.contains_key(&trace_id) {
                    EntryActor::cancel_downstream(trace_id, state);
                    state.pending_requests.remove(&trace_id);
                    tracing::info!(trace_id = %trace_id, "Canceled request after downstream signal");
                } else {
                    tracing::warn!(
                        trace_id = %trace_id,
                        "Cannot cancel unknown request"
                    );
                }
            }
            EntryMsg::CancelAll => {
                tracing::info!(
                    count = state.pending_requests.len(),
                    "Canceling all pending requests"
                );

                let snapshots: Vec<(Uuid, ActorRef<ExpertMsg>)> = state
                    .pending_requests
                    .iter()
                    .filter_map(|(id, r)| Some((*id, r.expert.clone()?)))
                    .collect();

                for (_, expert) in &snapshots {
                    let _ = expert.cast(ExpertMsg::MeshSignal(MeshSignal::Cancel));
                }
                state.pending_requests.clear();
            }
        }
        Ok(())
    }
}

pub async fn spawn_entry_actor(
    registry: Option<ActorRef<RegistryMsg>>,
) -> Result<ActorRef<EntryMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, EntryActor::new(), registry).await?;
    Ok(actor_ref)
}

pub async fn spawn_entry_actor_with_timeout(
    registry: Option<ActorRef<RegistryMsg>>,
    timeout_secs: u64,
) -> Result<ActorRef<EntryMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) =
        Actor::spawn(None, EntryActor::with_timeout(timeout_secs), registry).await?;
    Ok(actor_ref)
}
