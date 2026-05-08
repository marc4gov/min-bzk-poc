//! **PIIStripperExpert** — PII-maskering via LLM (Ollama), **[openai/privacy-filter]** sidecar,
//! of regex fallback.
//!
//! Opties (in volgorde van prioriteit):
//! 1. `MESH_PII_MODE=llm` — gebruik Ollama LLM met structured output (standaard)
//! 2. `PII_PRIVACY_FILTER_URL` — gebruik privacy-filter sidecar
//! 3. Regex fallback
//!
//! Geen volledige privacy-garantie.
//!
use ractor::{Actor, ActorProcessingErr, ActorRef};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::expert::{
    deliver_peer_response_result, send_work_output, BatonPass, ExpertMsg, ExpertState, PIICategory,
    PeerMap, WorkPayload,
};
use crate::mesh::types::MeshSignal;
use crate::mesh::{
    live::emit_mesh,
    ollama_bridge::{chat_via_ollama_for_mesh_with_events, mesh_ollama_enabled},
    privacy_filter_bridge::{self, PrivacyMaskResponse},
};

const PII_CAPABILITIES: &[&str] = &["pii", "anonymize", "scrub"];

const ENV_PII_MODE: &str = "MESH_PII_MODE";

#[derive(Debug, Clone, PartialEq)]
pub enum PIIMode {
    LLM,
    PrivacyFilter,
    Regex,
}

#[derive(Debug, Clone)]
pub struct PIIResult {
    pub scrubbed_content: String,
    pub report: HashMap<String, Vec<String>>,
}

/// LLM response struct for PII detection
#[derive(Debug, Deserialize)]
struct LLMPIIResponse {
    /// Whether PII was found
    #[allow(dead_code)]
    found: bool,
    /// List of detected PII entities
    #[serde(default)]
    entities: Vec<LLMPIIEntity>,
    /// Content with PII redacted
    masked: String,
}

#[derive(Debug, Deserialize, Clone)]
struct LLMPIIEntity {
    /// Type of PII (email, phone, name, address, etc.)
    #[serde(alias = "type")]
    entity_type: String,
    /// The original text that was detected
    original: String,
    /// Start position in original text (optional, for debugging)
    #[serde(default)]
    start: Option<usize>,
    /// End position in original text (optional)
    #[serde(default)]
    end: Option<usize>,
}

pub struct PIIStripperExpert {
    base: ExpertState,
    patterns: HashMap<PIICategory, regex::Regex>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl PIIStripperExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "PIIStripperExpert".to_string(),
            PII_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut patterns = HashMap::new();

        patterns.insert(
            PIICategory::Email,
            regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
        );
        patterns.insert(
            PIICategory::PhoneNumber,
            regex::Regex::new(r"\+?\d{1,3}[-.\s]?\(?\d{1,4}\)?[-.\s]?\d{1,4}[-.\s]?\d{1,9}")
                .unwrap(),
        );
        patterns.insert(
            PIICategory::SSN,
            regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b|\b\d{3}\s\d{2}\s\d{4}\b").unwrap(),
        );
        patterns.insert(
            PIICategory::IBAN,
            regex::Regex::new(r"[A-Z]{2}\d{2}[A-Z0-9]{11,30}").unwrap(),
        );
        patterns.insert(
            PIICategory::Name,
            regex::Regex::new(r"\b[A-Z][a-z]+\s[A-Z][a-z]+\b").unwrap(),
        );
        patterns.insert(
            PIICategory::Address,
            regex::Regex::new(
                r"\d+\s+[A-Z][a-z]+\s+(Street|St|Avenue|Ave|Road|Rd|Lane|Ln|Boulevard|Blvd)\b",
            )
            .unwrap(),
        );

        Self {
            base,
            patterns,
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

    /// Bepaal welke PII modus gebruikt moet worden
    fn pii_mode() -> PIIMode {
        // Check environment variable first
        match std::env::var(ENV_PII_MODE) {
            Ok(mode) => {
                match mode.to_lowercase().as_str() {
                    "llm" => PIIMode::LLM,
                    "privacy_filter" | "privacy-filter" | "hf" => PIIMode::PrivacyFilter,
                    "regex" => PIIMode::Regex,
                    _ => PIIMode::LLM, // default
                }
            }
            Err(_) => {
                // Default: use LLM if Ollama is available
                if mesh_ollama_enabled() {
                    PIIMode::LLM
                } else if privacy_filter_bridge::privacy_model_scrub_enabled() {
                    PIIMode::PrivacyFilter
                } else {
                    PIIMode::Regex
                }
            }
        }
    }

    pub(crate) fn category_labels(categories: &[PIICategory]) -> Vec<String> {
        categories
            .iter()
            .map(|c| match c {
                PIICategory::Email => "Email".to_string(),
                PIICategory::PhoneNumber => "PhoneNumber".to_string(),
                PIICategory::SSN => "SSN".to_string(),
                PIICategory::IBAN => "IBAN".to_string(),
                PIICategory::Name => "Name".to_string(),
                PIICategory::Address => "Address".to_string(),
                PIICategory::Custom(_) => "Custom".to_string(),
            })
            .collect()
    }

    /// Build PII detection prompt for LLM
    fn build_pii_prompt(content: &str, categories: &[PIICategory]) -> String {
        let cat_desc: Vec<String> = categories
            .iter()
            .map(|c| match c {
                PIICategory::Email => "email addresses".to_string(),
                PIICategory::PhoneNumber => "phone numbers".to_string(),
                PIICategory::SSN => "social security numbers".to_string(),
                PIICategory::IBAN => "IBAN bank account numbers".to_string(),
                PIICategory::Name => "person names".to_string(),
                PIICategory::Address => "physical addresses".to_string(),
                PIICategory::Custom(s) => s.clone(),
            })
            .collect();

        format!(
            "Je bent een PII (Privacy Information) detector. Analyseer de volgende tekst op: {}.\n\n\
            TEKST:\n```\n{}\n```\n\n\
            Reageer ALLEEN met geldige JSON in dit formaat (geen markdown, geen extra tekst):\n\
            {{\n\
              \"found\": true/false,\n\
              \"entities\": [\n\
                {{\"type\": \"email|phone|name|address|ssn|iban|other\", \"original\": \"gevonden tekst\"}}\n\
              ],\n\
              \"masked\": \"tekst met gevonden PII vervangen door [REDACTED]\"\n\
            }}\n\
            Wees nauwkeurig - maskeer alleen daadwerkelijke PII, geen algemene woorden.",
            cat_desc.join(", "),
            content
        )
    }

    /// PII detection using LLM
    async fn scrub_llm(&mut self, content: &str, cats: &[PIICategory]) -> Result<PIIResult, String> {
        let prompt = Self::build_pii_prompt(content, cats);

        match chat_via_ollama_for_mesh_with_events(
            "PIIStripperExpert",
            &prompt,
            None,
            &self.mesh_events,
        )
        .await
        {
            Ok(response) => {
                // Try to parse JSON from response
                let cleaned = response.trim().trim_start_matches("```json").trim_start_matches("```").trim();

                match serde_json::from_str::<LLMPIIResponse>(cleaned) {
                    Ok(llm_result) => {
                        let mut report: HashMap<String, Vec<String>> = HashMap::new();
                        for entity in &llm_result.entities {
                            report
                                .entry(entity.entity_type.clone())
                                .or_default()
                                .push(entity.original.clone());
                        }

                        Ok(PIIResult {
                            scrubbed_content: llm_result.masked,
                            report,
                        })
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, response = %cleaned, "LLM PII JSON parse failed");
                        Err(format!("JSON parse fout: {}. LLM antwoord: {}", e, cleaned.chars().take(200).collect::<String>()))
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "LLM PII request failed");
                Err(format!("LLM aanvraag mislukt: {}", e))
            }
        }
    }

    /// Format LLM report
    fn format_llm_report(report_no: u64, result: &PIIResult) -> String {
        let mut response = format!("PII Stripper Report #{} (LLM):\n", report_no);
        response.push_str("--- Model hits ---\n");
        for (category, matches) in &result.report {
            for m in matches {
                response.push_str(&format!("- {}: {}\n", category, m));
            }
        }
        response.push_str(&format!(
            "\nScrubbed content:\n{}\n",
            result.scrubbed_content
        ));
        response
    }

    fn scrub_regex_masks(&mut self, content: &str, categories: &[PIICategory]) -> PIIResult {
        let mut scrubbed_content = content.to_string();
        let mut report: HashMap<String, Vec<String>> = HashMap::new();

        for category in categories {
            let category_name = format!("{:?}", category);

            if let Some(pattern) = self.patterns.get(category) {
                let matches: Vec<String> = pattern
                    .find_iter(&scrubbed_content)
                    .map(|m| m.as_str().to_string())
                    .collect();

                if !matches.is_empty() {
                    report.insert(category_name.clone(), matches.clone());

                    scrubbed_content = pattern
                        .replace_all(&scrubbed_content, "[REDACTED]")
                        .to_string();
                }
            }
        }

        PIIResult {
            scrubbed_content,
            report,
        }
    }

    fn format_regex_report(report_no: u64, result: &PIIResult) -> String {
        let mut response = format!("PII Stripper Report #{} (regex):\n", report_no);
        for (category, matches) in &result.report {
            response.push_str(&format!(
                "- {}: {} instance(s) found and redacted\n",
                category,
                matches.len()
            ));
        }
        response.push_str(&format!(
            "\nScrubbed content:\n{}\n",
            result.scrubbed_content
        ));
        response
    }

    fn format_model_report(report_no: u64, out: &PrivacyMaskResponse) -> String {
        let mut response = format!(
            "PII Stripper Report #{} (openai/privacy-filter sidecar):\n",
            report_no
        );
        response.push_str("--- Model hits ---\n");
        for hit in &out.entities {
            let sc = hit
                .score
                .map(|x| format!(" score={:.3}", x))
                .unwrap_or_default();
            let w = hit
                .word
                .as_ref()
                .map(|x| format!(" word={:?}", x))
                .unwrap_or_default();
            response.push_str(&format!("- {}{}{}\n", hit.entity_group, sc, w));
        }
        response.push_str(&format!("\nScrubbed content:\n{}\n", out.masked));
        response
    }

    async fn scrub_payload(&mut self, content: &str, cats: &[PIICategory]) -> String {
        self.processed_count += 1;
        let n = self.processed_count;

        let mode = Self::pii_mode();

        match mode {
            PIIMode::LLM => {
                match self.scrub_llm(content, cats).await {
                    Ok(result) => {
                        tracing::info!(
                            scrub_no = n,
                            mode = "llm",
                            hits = result.report.len(),
                            "PII scrub succesvol"
                        );
                        Self::format_llm_report(n, &result)
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            scrub_no = n,
                            "LLM PII detection failed; attempting privacy-filter"
                        );
                        emit_mesh(
                            &self.mesh_events,
                            &self.base.name,
                            "pii_fallback_privacy_filter",
                            &format!("LLM gefaald: {}", e.chars().take(100).collect::<String>()),
                        );

                        // Try privacy-filter as fallback
                        let labels = Self::category_labels(cats);
                        let req = privacy_filter_bridge::PrivacyMaskRequest {
                            content,
                            categories: labels,
                        };

                        match privacy_filter_bridge::mask_via_service(req).await {
                            Ok(resp) => {
                                tracing::info!(scrub_no = n, "privacy-filter fallback succesvol");
                                Self::format_model_report(n, &resp)
                            }
                            Err(e2) => {
                                tracing::warn!(error = %e2, scrub_no = n, "privacy-filter ook gefaald; final fallback regex");
                                emit_mesh(
                                    &self.mesh_events,
                                    &self.base.name,
                                    "pii_fallback_regex",
                                    &format!("Privacy-filter gefaald: {}", e2.chars().take(100).collect::<String>()),
                                );
                                let fallback = self.scrub_regex_masks(content, cats);
                                Self::format_regex_report(n, &fallback)
                            }
                        }
                    }
                }
            }
            PIIMode::PrivacyFilter => {
                let labels = Self::category_labels(cats);
                let req = privacy_filter_bridge::PrivacyMaskRequest {
                    content,
                    categories: labels,
                };
                match privacy_filter_bridge::mask_via_service(req).await {
                    Ok(resp) => {
                        tracing::info!(
                            scrub_no = n,
                            hits = resp.entities.len(),
                            "privacy-filter model scrub"
                        );
                        Self::format_model_report(n, &resp)
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            scrub_no = n,
                            "privacy-filter service failed; fallback regex"
                        );
                        emit_mesh(
                            &self.mesh_events,
                            &self.base.name,
                            "pii_fallback_regex",
                            &format!("{}", e.chars().take(120).collect::<String>()),
                        );
                        let fallback = self.scrub_regex_masks(content, cats);
                        Self::format_regex_report(n, &fallback)
                    }
                }
            }
            PIIMode::Regex => {
                let r = self.scrub_regex_masks(content, cats);
                Self::format_regex_report(n, &r)
            }
        }
    }
}

#[async_trait::async_trait]
impl Actor for PIIStripperExpert {
    type Msg = ExpertMsg;
    type State = PIIStripperExpert;
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

                let mesh_detail = match Self::pii_mode() {
                    PIIMode::LLM => "LLM PII-detectie via Ollama (+ fallback naar privacy-filter/regex bij fout)",
                    PIIMode::PrivacyFilter => "privacy-filter model (+ fallback regex bij fout)",
                    PIIMode::Regex => "regex PII-maskering",
                };

                let result = if let WorkPayload::ScrubPII {
                    content,
                    pii_categories,
                } = &envelope.payload
                {
                    state
                        .scrub_payload(content.as_str(), pii_categories.as_slice())
                        .await
                } else {
                    "PIIStripperExpert: Please provide ScrubPII payload with content and categories"
                        .to_string()
                };

                emit_mesh(
                    &state.mesh_events,
                    &state.base.name,
                    "scrub_klaar",
                    mesh_detail,
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

pub async fn spawn_pii_stripper_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, PIIStripperExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_pii_stripper_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) =
        Actor::spawn(None, PIIStripperExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_pii_stripper_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(
        None,
        PIIStripperExpert::with_timeout(timeout_secs),
        (None, None),
    )
    .await?;
    Ok(actor_ref)
}
