use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::mesh::types::RegistryMsg;

const HEARTBEAT_TIMEOUT_SECS: u64 = 30;

pub struct RegistryActor {
    capabilities: HashMap<String, Vec<ActorRef<RegistryMsg>>>,
    health_map: HashMap<String, Instant>,
    heartbeat_timeout_secs: u64,
}

impl RegistryActor {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            health_map: HashMap::new(),
            heartbeat_timeout_secs: HEARTBEAT_TIMEOUT_SECS,
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            capabilities: HashMap::new(),
            health_map: HashMap::new(),
            heartbeat_timeout_secs: timeout_secs,
        }
    }

    fn cleanup_stale_actors(&mut self) -> Vec<String> {
        let now = Instant::now();
        let timeout = Duration::from_secs(self.heartbeat_timeout_secs);

        let stale_ids: Vec<String> = self
            .health_map
            .iter()
            .filter(|(_, last_seen)| now.duration_since(**last_seen) > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &stale_ids {
            self.remove_actor(id);
        }

        if !stale_ids.is_empty() {
            tracing::debug!(
                count = stale_ids.len(),
                active = self.health_map.len(),
                "Health monitor: removed stale actors"
            );
        }

        stale_ids
    }

    fn remove_actor(&mut self, actor_id: &str) {
        tracing::info!(
            actor_id = %actor_id,
            "Removing stale actor from registry"
        );

        self.health_map.remove(actor_id);

        for actors in self.capabilities.values_mut() {
            actors.retain(|actor| actor.get_id().to_string() != actor_id);
        }

        self.capabilities.retain(|_, actors| !actors.is_empty());
    }

    fn handle_register(
        &mut self,
        actor: ActorRef<RegistryMsg>,
        capability: String,
        _metadata: HashMap<String, String>,
    ) {
        let actor_id = actor.get_id().to_string();

        self.health_map.insert(actor_id.clone(), Instant::now());

        self.capabilities
            .entry(capability.clone())
            .or_insert_with(Vec::new)
            .push(actor.clone());

        tracing::debug!(
            capability = %capability,
            actor_id = %actor_id,
            "Actor registered capability"
        );
    }

    fn handle_resolve_capability(&mut self, capability: String) -> Vec<ActorRef<RegistryMsg>> {
        self.cleanup_stale_actors();

        self.capabilities
            .get(&capability)
            .cloned()
            .unwrap_or_default()
    }

    fn handle_heartbeat(&mut self, actor: ActorRef<RegistryMsg>) {
        let actor_id = actor.get_id().to_string();
        self.health_map.insert(actor_id.clone(), Instant::now());
        tracing::trace!(actor_id = %actor_id, "Heartbeat received");
    }
}

#[async_trait::async_trait]
impl Actor for RegistryActor {
    type Msg = RegistryMsg;
    type State = RegistryActor;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(RegistryActor::new())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            RegistryMsg::Register {
                actor,
                capability,
                metadata,
            } => {
                if let Some(actor_ref) = actor {
                    state.handle_register(actor_ref, capability, metadata);
                }
            }
            RegistryMsg::ResolveCapability {
                capability,
                reply_to,
            } => {
                let actors = state.handle_resolve_capability(capability.clone());

                if let Some(reply_to_ref) = reply_to {
                    for _actor_ref in actors {
                        let _ = reply_to_ref.cast(RegistryMsg::ResolveCapability {
                            capability: capability.clone(),
                            reply_to: None,
                        });
                    }
                }
            }
            RegistryMsg::Heartbeat { actor } => {
                if let Some(actor_ref) = actor {
                    state.handle_heartbeat(actor_ref);
                }
            }
        }
        Ok(())
    }
}

pub async fn spawn_registry() -> Result<ActorRef<RegistryMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, RegistryActor::new(), ()).await?;
    Ok(actor_ref)
}
