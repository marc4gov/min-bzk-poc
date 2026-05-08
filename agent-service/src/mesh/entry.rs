use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use uuid::Uuid;

use crate::mesh::expert::{ExpertMsg, WorkEnvelope, WorkPayload};
use crate::mesh::research_http;
use crate::mesh::types::{EntryMsg, MeshSignal, RegistryMsg};

const MAX_PENDING_REQUESTS: usize = 100;
const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// Triage op woorden zoals „rust”; URL’s worden genegeerd (`rust-lang.org` ⇒ geen `rust`-deskundige).
fn text_without_urls_lower(query: &str) -> String {
    let mut t = query.to_string();
    for u in research_http::extract_http_urls_from_text(query) {
        t = t.replace(&u, " ");
    }
    t.to_lowercase()
}

fn has_document_pipeline_intent_text(q_lower: &str) -> bool {
    const INTENT: &[&str] = &[
        "samenvat",
        "summar",
        "rapport",
        "verslag",
        "document",
        "brief",
        "schrijf",
        "maak ",
        "maak een",
        "genereer",
        "create ",
        "draft ",
        "publicatie",
        "stakeholder",
        "officie",
        "formele",
        "formeel",
        "memo",
        "board deck",
        "voor het team",
        "for the team",
        "uitleggen aan",
        "meetwaardig",
        "explain to management",
        "report",
        "summary ",
    ];
    INTENT.iter().any(|t| q_lower.contains(t))
}

fn connector_phrases_suggest_pipeline(q_lower: &str) -> bool {
    q_lower.contains("op basis van")
        || q_lower.contains("gebaseerd op")
        || q_lower.contains("based on")
        || q_lower.contains("using this ")
        || q_lower.contains("van deze pagina")
        || q_lower.contains("from this url")
        || q_lower.contains("lees deze ")
        || q_lower.contains("read this page")
        || q_lower.contains("gebruik deze link")
}

/// Minstens één http(s)-URL én formulering die richting de meerstaps **CreateDocument**-workflow wijst
/// (bv. rapportage, samenvatting voor anderen, verbonden aan “op basis van …”).
///
/// Bewust geen alleen‑URL‑match om “wat staat er op deze site?” (lookup) nog naar `research`/`general` te laten gaan.
fn infer_document_pipeline_from_natural_query(query: &str) -> bool {
    if research_http::extract_http_urls_from_text(query).is_empty() {
        return false;
    }
    let q = query.trim().to_lowercase();

    // Korte informatieve lookup zonder tekst‑output‑intent (“wat is deze site”).
    let t = q.trim_start();
    if (t.starts_with("wat is ")
        || t.starts_with("wie is ")
        || t.starts_with("what is ")
        || t.starts_with("who is ")
        || t.starts_with("wat zijn ")
        || t.starts_with("where is "))
        && !has_document_pipeline_intent_text(&q)
        && !connector_phrases_suggest_pipeline(&q)
    {
        return false;
    }

    connector_phrases_suggest_pipeline(&q) || (has_document_pipeline_intent_text(&q))
}

/// Zelfde triage-logica als [`EntryActor`] — gebruikt door mesh‑demo (`full_mesh`) voor document-route.
#[must_use]
pub fn triage_entry_capability(query: &str) -> String {
    if infer_document_pipeline_from_natural_query(query) {
        return "document".to_string();
    }

    let query_lower = query.to_lowercase();
    let no_url = text_without_urls_lower(query);

    if no_url.contains("rust")
        || no_url.contains("memory")
        || no_url.contains("thread")
        || no_url.contains("wasm")
    {
        "rust".to_string()
    } else if no_url.contains("frontend") || no_url.contains("ui") || no_url.contains("react") {
        "frontend".to_string()
    } else if no_url.contains("database") || query_lower.contains("sql") {
        "database".to_string()
    } else if no_url.contains("research") || no_url.contains("scrape") || no_url.contains("gather")
    {
        "research".to_string()
    } else if no_url.contains("write") || no_url.contains("draft") || no_url.contains("compose") {
        "write".to_string()
    } else if no_url.contains("pii") || no_url.contains("anonymize") || no_url.contains("scrub") {
        "pii".to_string()
    } else if no_url.contains("review") || no_url.contains("critique") || no_url.contains("edit") {
        "review".to_string()
    } else if query_lower.contains("document") || query_lower.contains("create doc") {
        "document".to_string()
    } else {
        "general".to_string()
    }
}

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
        triage_entry_capability(query)
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
            let _ = reply_to_ref.cast(EntryMsg::ExpertResponse { trace_id, result });
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
            EntryMsg::SubmitRequest {
                query,
                context,
                reply_to,
            } => {
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
            EntryMsg::SubmitDocument {
                filename,
                content,
                instructions,
                context,
                reply_to,
            } => {
                state.cleanup_expired_requests();

                if state.pending_requests.len() >= MAX_PENDING_REQUESTS {
                    tracing::warn!("Too many pending requests, rejecting document upload");
                    EntryActor::deliver_final_response(
                        Uuid::new_v4(),
                        "Error: Too many pending requests".to_string(),
                        reply_to,
                    );
                    return Ok(());
                }

                let trace_id = Uuid::new_v4();

                tracing::info!(
                    trace_id = %trace_id,
                    filename = %filename,
                    content_len = content.len(),
                    "Document upload submitted"
                );

                // Route naar document-improve expert
                let expert = state.find_expert_for_capability("improve-doc");

                if let Some(expert_ref) = expert {
                    let mut request = PendingRequest::new(trace_id, context.clone(), reply_to);
                    request.expert = Some(expert_ref.clone());

                    let work_envelope = WorkEnvelope {
                        payload: WorkPayload::ImproveDocument {
                            content,
                            filename,
                            instructions,
                            pii_categories: vec![],
                        },
                        context,
                        reply_to: None,
                        entry_reply: Some(myself.clone()),
                        trace_id,
                        hop_count: 0,
                    };

                    let _ = expert_ref.cast(ExpertMsg::Work(work_envelope));
                    state.pending_requests.insert(trace_id, request);
                } else {
                    tracing::warn!("No expert found for improve-doc capability");
                    EntryActor::deliver_final_response(
                        trace_id,
                        "Error: Document verbetering niet beschikbaar".to_string(),
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

#[cfg(test)]
mod triage_tests {
    use super::triage_entry_capability;

    #[test]
    fn triage_priorities_rust_over_react_keyword_order() {
        assert_eq!(triage_entry_capability("rust and react hooks"), "rust");
    }

    #[test]
    fn triage_document() {
        assert_eq!(
            triage_entry_capability("Create document from notes"),
            "document"
        );
        assert_eq!(
            triage_entry_capability("needs a document revision"),
            "document"
        );
    }

    #[test]
    fn triage_research_before_document_when_both_terms() {
        assert_eq!(
            triage_entry_capability("research document corpus on climate"),
            "research"
        );
    }

    #[test]
    fn nl_vraag_met_url_en_samenvatting_routes_naar_document() {
        assert_eq!(
            triage_entry_capability(
                "Kun je op basis van https://www.rust-lang.org een korte samenvatting voor het team schrijven?"
            ),
            "document"
        );
    }

    #[test]
    fn puur_informatieve_lookup_wat_is_geen_document() {
        assert_ne!(
            triage_entry_capability("Wat is https://example.org?"),
            "document"
        );
    }
}
