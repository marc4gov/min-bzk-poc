use ractor::ActorRef;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: Uuid,
    pub user_id: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<M> {
    pub payload: M,
    pub context: SessionContext,
    #[serde(skip)]
    pub reply_to: Option<ActorRef<M>>,
    pub trace_id: Uuid,
    pub hop_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryMsg {
    Register {
        #[serde(skip)]
        actor: Option<ActorRef<RegistryMsg>>,
        capability: String,
        metadata: std::collections::HashMap<String, String>,
    },
    ResolveCapability {
        capability: String,
        #[serde(skip)]
        reply_to: Option<ActorRef<RegistryMsg>>,
    },
    Heartbeat {
        #[serde(skip)]
        actor: Option<ActorRef<RegistryMsg>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshSignal {
    Cancel,
}

impl<M> Envelope<M> {
    pub fn new(payload: M, context: SessionContext, reply_to: Option<ActorRef<M>>) -> Self {
        Self {
            payload,
            context,
            reply_to,
            trace_id: Uuid::new_v4(),
            hop_count: 0,
        }
    }
}

// Since ActorRef doesn't implement Default, we can't use derive(Deserialize)
// on types containing it if we use #[serde(skip)] unless we provide a custom
// deserialization method or wrap it in an Option.
// However, for these core mesh types, the ActorRef is typically not
// serialized/deserialized across the network in the same way as the payload.
// If they are, we would use a unique identifier (like a UUID) instead of the ActorRef itself.
