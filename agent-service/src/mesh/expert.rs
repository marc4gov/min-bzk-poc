use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::mesh::types::{MeshSignal, SessionContext};

const MAX_HOPS: u32 = 5;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const TIMER_INTERVAL_SECS: u64 = 10;

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
    TimerTick,
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
    pub recovery_attempted: bool,
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
            recovery_attempted: false,
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
    pub timer_enabled: bool,
}

impl ExpertState {
    pub fn new(name: String, capabilities: Vec<String>) -> Self {
        Self {
            name,
            capabilities,
            pending_tasks: HashMap::new(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            timer_enabled: true,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_timer(mut self, enabled: bool) -> Self {
        self.timer_enabled = enabled;
        self
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    pub fn cleanup_expired_tasks(&mut self) -> Vec<Uuid> {
        let now = Instant::now();
        let timeout = Duration::from_secs(self.timeout_secs);

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

    pub fn recover_expired_tasks(&mut self, myself: &ActorRef<ExpertMsg>) -> Vec<Uuid> {
        let now = Instant::now();
        let timeout = Duration::from_secs(self.timeout_secs);

        let expired: Vec<(Uuid, PendingTask)> = self
            .pending_tasks
            .iter()
            .filter(|(_, task)| {
                now.duration_since(task.created_at) > timeout && !task.recovery_attempted
            })
            .map(|(id, task)| (*id, task.clone()))
            .collect();

        let mut recovered = Vec::new();

        for (trace_id, task) in expired {
            tracing::warn!(
                trace_id = %trace_id,
                capability = %task.capability_requested,
                "Pending task expired, initiating recovery"
            );

            if let Some(ref reply_to) = task.reply_to {
                let _ = reply_to.cast(ExpertMsg::PeerResponse {
                    trace_id,
                    result: format!("Timeout: Peer at '{}' did not respond for capability '{}'",
                        task.capability_requested, self.name),
                });
            }

            if let Some(ref peer) = task.peer {
                tracing::info!(
                    trace_id = %trace_id,
                    "Sending cancel signal to unresponsive peer"
                );
                let _ = peer.cast(ExpertMsg::MeshSignal(MeshSignal::Cancel));
            }

            let mut updated_task = task.clone();
            updated_task.recovery_attempted = true;
            self.pending_tasks.insert(trace_id, updated_task);

            recovered.push(trace_id);
        }

        recovered
    }

    pub fn purge_expired_tasks(&mut self) -> Vec<PendingTask> {
        let now = Instant::now();
        let timeout = Duration::from_secs(self.timeout_secs);
        let recovery_timeout = Duration::from_secs(self.timeout_secs + 5);

        let purged: Vec<PendingTask> = self
            .pending_tasks
            .iter()
            .filter(|(_, task)| {
                let elapsed = now.duration_since(task.created_at);
                elapsed > recovery_timeout || (elapsed > timeout && task.recovery_attempted)
            })
            .map(|(_, task)| task.clone())
            .collect();

        for task in &purged {
            self.pending_tasks.remove(&task.trace_id);
        }

        purged
    }

    pub fn propagate_poison_pill(&mut self, trace_id: Option<Uuid>) {
        tracing::info!(
            expert = %self.name,
            pending = self.pending_tasks.len(),
            trace_id = ?trace_id,
            "Propagating poison pill (cancel signal)"
        );

        if let Some(id) = trace_id {
            if let Some(task) = self.pending_tasks.get(&id) {
                if let Some(ref peer) = task.peer {
                    tracing::debug!(
                        trace_id = %id,
                        peer = ?peer,
                        "Sending cancel to peer"
                    );
                    let _ = peer.cast(ExpertMsg::MeshSignal(MeshSignal::Cancel));
                }
            }
            self.pending_tasks.remove(&id);
        } else {
            for (trace_id, task) in &self.pending_tasks {
                if let Some(ref peer) = task.peer {
                    let _ = peer.cast(ExpertMsg::MeshSignal(MeshSignal::Cancel));
                }
            }
            self.pending_tasks.clear();
        }
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

    pub fn create_pending_task_with_peer(
        envelope: &WorkEnvelope,
        capability_requested: String,
        peer: ActorRef<ExpertMsg>,
    ) -> PendingTask {
        let mut task = PendingTask::new(
            envelope.trace_id,
            envelope.context.clone(),
            envelope.reply_to.clone(),
            capability_requested,
            envelope.hop_count,
        );
        task.peer = Some(peer);
        task
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
        myself: ActorRef<Self::Msg>,
        args: (String, Vec<String>),
    ) -> Result<Self::State, ActorProcessingErr> {
        let (name, capabilities) = args;
        let mut state = ExpertState::new(name, capabilities);

        if state.timer_enabled {
            let myself_clone = myself.clone();
            let interval = Duration::from_secs(TIMER_INTERVAL_SECS);

            tokio::spawn(async move {
                let mut timer = tokio::time::interval(interval);
                loop {
                    timer.tick().await;
                    let _ = myself_clone.cast(ExpertMsg::TimerTick);
                }
            });

            tracing::debug!(
                timer_interval_secs = TIMER_INTERVAL_SECS,
                "Stability timer started"
            );
        }

        Ok(state)
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
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
                    state.propagate_poison_pill(None);
                }
            },
            ExpertMsg::TimerTick => {
                if state.timer_enabled {
                    let recovered = state.recover_expired_tasks(&_myself);
                    let purged = state.purge_expired_tasks();

                    if !recovered.is_empty() || !purged.is_empty() {
                        tracing::debug!(
                            expert = %state.name,
                            recovered = recovered.len(),
                            purged = purged.len(),
                            remaining = state.pending_tasks.len(),
                            "Timer tick: stability maintenance"
                        );
                    }
                }
            }
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
