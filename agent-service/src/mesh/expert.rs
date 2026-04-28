use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

use crate::mesh::types::{MeshSignal, SessionContext};

const MAX_HOPS: u32 = 5;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub enum WorkPayload {
    Query(String),
    Process(String),
    Delegate { capability: String, payload: String },
}

#[derive(Debug, Clone)]
pub struct WorkEnvelope {
    pub payload: WorkPayload,
    pub context: SessionContext,
    pub reply_to: Option<ActorRef<ExpertMsg>>,
    pub trace_id: Uuid,
    pub hop_count: u32,
}

#[derive(Debug, Clone)]
pub enum ExpertMsg {
    Work(WorkEnvelope),
    PeerResponse {
        trace_id: Uuid,
        result: String,
    },
    MeshSignal(MeshSignal),
}

#[derive(Debug, Clone)]
pub struct PendingTask {
    pub trace_id: Uuid,
    pub original_context: SessionContext,
    pub reply_to: Option<ActorRef<ExpertMsg>>,
    pub peer: Option<ActorRef<ExpertMsg>>,
    pub created_at: Instant,
    pub hop_count: u32,
    pub capability_requested: String,
}

impl PendingTask {
    pub fn new(
        trace_id: Uuid,
        original_context: SessionContext,
        reply_to: Option<ActorRef<ExpertMsg>>,
        capability_requested: String,
        hop_count: u32,
    ) -> Self {
        Self {
            trace_id,
            original_context,
            reply_to,
            peer: None,
            created_at: Instant::now(),
            hop_count,
            capability_requested,
        }
    }

    pub fn is_expired(&self, timeout_secs: u64) -> bool {
        self.created_at.elapsed().as_secs() > timeout_secs
    }

    pub fn hops_exceeded(&self) -> bool {
        self.hop_count >= MAX_HOPS
    }
}

#[derive(Debug, Clone)]
pub enum TaskState {
    Processing(String),
    Delegated(String, ActorRef<ExpertMsg>),
    Completed(String),
    Failed(String),
}

pub struct ExpertState {
    pub name: String,
    pub capabilities: Vec<String>,
    pub pending_tasks: HashMap<Uuid, PendingTask>,
    pub timeout_secs: u64,
}

impl ExpertState {
    pub fn new(name: String, capabilities: Vec<String>) -> Self {
        Self {
            name,
            capabilities,
            pending_tasks: HashMap::new(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    pub fn cleanup_expired_tasks(&mut self) -> Vec<Uuid> {
        let now = Instant::now();
        let timeout = std::time::Duration::from_secs(self.timeout_secs);

        let expired: Vec<Uuid> = self
            .pending_tasks
            .iter()
            .filter(|(_, task)| now.duration_since(task.created_at) > timeout)
            .map(|(id, _)| *id)
            .collect();

        for id in &expired {
            self.pending_tasks.remove(id);
        }

        expired
    }

    pub fn can_delegate(&self, hop_count: u32) -> bool {
        hop_count < MAX_HOPS
    }

    pub fn store_pending_task(&mut self, task: PendingTask) {
        self.pending_tasks.insert(task.trace_id, task);
    }
}

pub struct BatonPass;

impl BatonPass {
    pub fn prepare_delegation(
        envelope: &WorkEnvelope,
        _peer: ActorRef<ExpertMsg>,
        new_capability: String,
    ) -> WorkEnvelope {
        WorkEnvelope {
            payload: WorkPayload::Delegate {
                capability: new_capability.clone(),
                payload: format!("{:?}", envelope.payload),
            },
            context: envelope.context.clone(),
            reply_to: None,
            trace_id: envelope.trace_id,
            hop_count: envelope.hop_count + 1,
        }
    }

    pub fn create_pending_task(
        envelope: &WorkEnvelope,
        capability_requested: String,
    ) -> PendingTask {
        PendingTask::new(
            envelope.trace_id,
            envelope.context.clone(),
            envelope.reply_to.clone(),
            capability_requested,
            envelope.hop_count,
        )
    }

    pub fn respond_to_original(
        result: String,
        task: &PendingTask,
    ) -> ExpertMsg {
        ExpertMsg::PeerResponse {
            trace_id: task.trace_id,
            result,
        }
    }

    pub fn check_hop_limit(hop_count: u32) -> Result<(), ExpertError> {
        if hop_count >= MAX_HOPS {
            Err(ExpertError::MaxHopsReached(hop_count))
        } else {
            Ok(())
        }
    }

    pub fn create_response(trace_id: Uuid, result: String) -> ExpertMsg {
        ExpertMsg::PeerResponse {
            trace_id,
            result,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExpertError {
    #[error("Max hops exceeded: {0}")]
    MaxHopsReached(u32),

    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),

    #[error("Task expired: {0}")]
    TaskExpired(Uuid),

    #[error("Peer timeout: {0}")]
    PeerTimeout(Uuid),
}

#[async_trait::async_trait]
impl Actor for ExpertState {
    type Msg = ExpertMsg;
    type State = ExpertState;
    type Arguments = (String, Vec<String>);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: (String, Vec<String>),
    ) -> Result<Self::State, ActorProcessingErr> {
        let (name, capabilities) = args;
        Ok(ExpertState::new(name, capabilities))
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ExpertMsg::Work(envelope) => {
                if let Err(e) = BatonPass::check_hop_limit(envelope.hop_count) {
                    tracing::warn!(
                        trace_id = %envelope.trace_id,
                        error = %e,
                        "Hop limit check failed"
                    );
                    if let Some(reply_to) = &envelope.reply_to {
                        let _ = reply_to.cast(BatonPass::create_response(
                            envelope.trace_id,
                            format!("Error: {}", e),
                        ));
                    }
                    return Ok(());
                }

                state.cleanup_expired_tasks();

                match &envelope.payload {
                    WorkPayload::Query(q) => {
                        tracing::info!(
                            expert = %state.name,
                            query = %q,
                            "Processing query"
                        );
                        if let Some(reply_to) = &envelope.reply_to {
                            let _ = reply_to.cast(BatonPass::create_response(
                                envelope.trace_id,
                                format!("Processed by {}: {}", state.name, q),
                            ));
                        }
                    }
                    WorkPayload::Process(p) => {
                        tracing::info!(
                            expert = %state.name,
                            payload = %p,
                            "Processing work payload"
                        );
                        if let Some(reply_to) = &envelope.reply_to {
                            let _ = reply_to.cast(BatonPass::create_response(
                                envelope.trace_id,
                                format!("Processed by {}: {}", state.name, p),
                            ));
                        }
                    }
                    WorkPayload::Delegate { capability, payload } => {
                        tracing::info!(
                            expert = %state.name,
                            capability = %capability,
                            payload = %payload,
                            "Delegation request"
                        );
                        if let Some(reply_to) = &envelope.reply_to {
                            let _ = reply_to.cast(BatonPass::create_response(
                                envelope.trace_id,
                                format!("Delegated to {} by {}", capability, state.name),
                            ));
                        }
                    }
                }
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.pending_tasks.remove(&trace_id) {
                    tracing::info!(
                        trace_id = %trace_id,
                        result = %result,
                        "Peer response received, forwarding to original caller"
                    );

                    if let Some(ref reply_to) = task.reply_to {
                        let response = BatonPass::respond_to_original(result, &task);
                        let _ = reply_to.cast(response);
                    }
                } else {
                    tracing::warn!(
                        trace_id = %trace_id,
                        "Received response for unknown task"
                    );
                }
            }
            ExpertMsg::MeshSignal(signal) => match signal {
                MeshSignal::Cancel => {
                    tracing::info!(
                        expert = %state.name,
                        "Cancel signal received, clearing pending tasks"
                    );
                    state.pending_tasks.clear();
                }
            },
        }
        Ok(())
    }
}

pub async fn spawn_expert(
    name: String,
    capabilities: Vec<String>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let args = (name.clone(), capabilities.clone());
    let (actor_ref, _) = Actor::spawn(None, ExpertState::new(name, capabilities), args).await?;
    Ok(actor_ref)
}
