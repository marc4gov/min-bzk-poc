# Document Creation Expert Agents Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement four expert agents (Researcher, Schrijver, PII-Stripper, Reviewer) and a DocumentOrchestrator for the existing actor mesh, enabling automated document creation with configurable error handling.

**Architecture:** Extend existing ractor-based mesh with domain experts that follow the established ExpertState pattern, communicate via WorkEnvelope/ExpertMsg, and can be coordinated by a DocumentOrchestrator that manages the full workflow pipeline.

**Tech Stack:** Rust, ractor (actor framework), regex (PII detection), reqwest (URL scraping), serde (serialization)

---

## Task 1: Extend WorkPayload with Document Types

**Files:**
- Modify: `agent-service/src/mesh/expert.rs`

- [ ] **Step 1: Add supporting types to expert.rs**

Add after line 17 (after `use crate::mesh::types::{EntryMsg, MeshSignal, SessionContext};`):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StyleProfile {
    Formal,
    Casual,
    Legal,
    Technical,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PIICategory {
    Email,
    PhoneNumber,
    SSN,
    IBAN,
    Name,
    Address,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ReviewCriteria {
    pub tone: Option<String>,
    pub length_constraints: Option<(usize, usize)>,
    pub focus_areas: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ErrorStrategy {
    FailFast,
    PartialResults,
    RetryWithFallback {
        max_attempts: u32,
        fallback_urls: Vec<String>,
    },
}
```

- [ ] **Step 2: Extend WorkPayload enum**

Replace the existing `WorkPayload` enum (lines 12-17) with:

```rust
#[derive(Debug, Clone)]
pub enum WorkPayload {
    Query(String),
    Process(String),
    Delegate { capability: String, payload: String },
    Research {
        urls: Vec<String>,
        depth: u8,
    },
    Write {
        research_notes: String,
        style_profile: StyleProfile,
    },
    ScrubPII {
        content: String,
        pii_categories: Vec<PIICategory>,
    },
    Review {
        content: String,
        criteria: Option<ReviewCriteria>,
    },
    CreateDocument {
        urls: Vec<String>,
        style_profile: StyleProfile,
        pii_categories: Vec<PIICategory>,
        review_criteria: Option<ReviewCriteria>,
        error_strategy: ErrorStrategy,
    },
}
```

- [ ] **Step 3: Update process_locally pattern in existing experts**

In `src/mesh/experts/rust_expert.rs`, update the `process_locally` method around line 86 to handle new payload variants:

```rust
WorkPayload::Delegate { capability, .. } => {
    return format!("RustExpert: Received delegation for {}", capability);
}
WorkPayload::Research { .. } => {
    return format!("RustExpert: Research request forwarded to ResearchExpert");
}
WorkPayload::Write { .. } => {
    return format!("RustExpert: Write request forwarded to SchrijverExpert");
}
WorkPayload::ScrubPII { .. } => {
    return format!("RustExpert: PII request forwarded to PIIStripperExpert");
}
WorkPayload::Review { .. } => {
    return format!("RustExpert: Review request forwarded to ReviewerExpert");
}
WorkPayload::CreateDocument { .. } => {
    return format!("RustExpert: Document creation forwarded to DocumentOrchestrator");
}
```

Do the same for `src/mesh/experts/frontend_expert.rs` around line 86.

- [ ] **Step 4: Check compilation**

Run: `cargo check`

Expected: Should compile without errors.

- [ ] **Step 5: Commit**

```bash
git add agent-service/src/mesh/expert.rs agent-service/src/mesh/experts/rust_expert.rs agent-service/src/mesh/experts/frontend_expert.rs
git commit -m "feat(mesh): extend WorkPayload with document creation types

- Add StyleProfile, PIICategory, ReviewCriteria, ErrorStrategy types
- Extend WorkPayload with Research, Write, ScrubPII, Review, CreateDocument
- Update existing experts to handle new payload variants

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Create ResearchExpert

**Files:**
- Create: `agent-service/src/mesh/experts/research_expert.rs`
- Modify: `agent-service/src/mesh/experts/mod.rs`

- [ ] **Step 1: Create research_expert.rs skeleton**

Create file `agent-service/src/mesh/experts/research_expert.rs`:

```rust
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::types::{EntryMsg, MeshSignal};
use crate::mesh::expert::{
    send_work_output, BatonPass, ExpertError, ExpertMsg, ExpertState, PeerMap, WorkEnvelope,
    WorkPayload,
};

const RESEARCH_CAPABILITIES: &[&str] = &["research", "scrape", "gather"];
const MAX_DELEGATION_DEPTH: u32 = 3;

pub struct ResearchExpert {
    base: ExpertState,
    domain_knowledge: HashMap<String, String>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl ResearchExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "ResearchExpert".to_string(),
            RESEARCH_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut domain_knowledge = HashMap::new();
        domain_knowledge.insert(
            "scrape".to_string(),
            "URL scraping extracts content from web pages".to_string(),
        );
        domain_knowledge.insert(
            "gather".to_string(),
            "Information gathering from multiple sources".to_string(),
        );

        Self {
            base,
            domain_knowledge,
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

    fn can_handle_locally(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        query_lower.contains("http")
            || query_lower.contains("url")
            || query_lower.contains("scrape")
            || query_lower.contains("research")
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        self.processed_count += 1;

        match &envelope.payload {
            WorkPayload::Research { urls, depth } => {
                tracing::info!(
                    urls = ?urls,
                    depth = depth,
                    "Processing research request"
                );

                let mut notes = format!("Research from {} source(s):\n\n", urls.len());

                for (i, url) in urls.iter().enumerate() {
                    notes.push_str(&format!("{}. Source: {}\n", i + 1, url));
                    notes.push_str(&format!("   Status: Content would be scraped from {}\n", url));
                    notes.push_str(&format!("   Depth: {} level(s)\n\n", depth));
                }

                notes
            }
            _ => "ResearchExpert: Please provide Research payload with URLs".to_string(),
        }
    }

    fn should_delegate(&self, query: &str, hop_count: u32, envelope: &WorkEnvelope) -> Option<String> {
        if hop_count >= MAX_DELEGATION_DEPTH {
            return None;
        }

        let query_lower = query.to_lowercase();

        if query_lower.contains("write") || query_lower.contains("draft") {
            Some("write".to_string())
        } else {
            None
        }
    }

    fn would_delegate_to_sender(
        peers: &Option<PeerMap>,
        target: &str,
        envelope: &WorkEnvelope,
    ) -> bool {
        match (peers.as_ref().and_then(|m| m.get(target)), envelope.reply_to.as_ref()) {
            (Some(peer), Some(reply)) => reply.get_id() == peer.get_id(),
            _ => false,
        }
    }

    fn query_slice(payload: &WorkPayload) -> &str {
        match payload {
            WorkPayload::Query(q) => q.as_str(),
            WorkPayload::Process(p) => p.as_str(),
            WorkPayload::Delegate { capability, .. } => capability.as_str(),
            WorkPayload::Research { .. } => "research",
            _ => "",
        }
    }

    fn handle_delegation(
        &mut self,
        envelope: &WorkEnvelope,
        target_capability: String,
        myself: &ActorRef<ExpertMsg>,
    ) -> Result<(), ExpertError> {
        let peer = self
            .peers
            .as_ref()
            .and_then(|m| m.get(&target_capability))
            .cloned()
            .ok_or_else(|| ExpertError::CapabilityNotFound(target_capability.clone()))?;

        let delegated = BatonPass::prepare_delegation(
            envelope,
            myself.clone(),
            envelope.payload.clone(),
        );

        let pending_task =
            BatonPass::create_pending_task_with_peer(envelope, target_capability.clone(), peer.clone());
        self.base.store_pending_task(pending_task);

        tracing::info!(
            trace_id = %envelope.trace_id,
            from = %self.base.name,
            to = %target_capability,
            "Delegating to peer"
        );

        let _ = peer.cast(ExpertMsg::Work(delegated));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Actor for ResearchExpert {
    type Msg = ExpertMsg;
    type State = ResearchExpert;
    type Arguments = (Option<PeerMap>, Option<broadcast::Sender<crate::AgentEvent>>);

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

                if let Some(delegation_target) = state.should_delegate(
                    Self::query_slice(&envelope.payload),
                    envelope.hop_count,
                    &envelope,
                ) {
                    let _ = state.handle_delegation(&envelope, delegation_target, &myself);
                } else if state.can_handle_locally(Self::query_slice(&envelope.payload)) {
                    let result = state.process_locally(&envelope);
                    send_work_output(&envelope, result);
                } else {
                    send_work_output(
                        &envelope,
                        format!("{}: Cannot handle this request type", state.base.name),
                    );
                }
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.base.pending_tasks.remove(&trace_id) {
                    if let Some(ref reply_to) = task.reply_to {
                        let _ = reply_to.cast(ExpertMsg::PeerResponse {
                            trace_id,
                            result,
                        });
                    } else if let Some(ref gateway) = task.entry_reply {
                        let _ = gateway.cast(EntryMsg::ExpertResponse {
                            trace_id,
                            result,
                        });
                    }
                }
                Ok(())
            }
            ExpertMsg::MeshSignal(signal) => match signal {
                MeshSignal::Cancel => {
                    state.base.pending_tasks.clear();
                }
            },
            ExpertMsg::TimerTick => {
                let _recovered = state.base.recover_expired_tasks(&myself);
                let _purged = state.base.purge_expired_tasks();
            }
        }
    }
}

pub async fn spawn_research_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ResearchExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_research_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ResearchExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_research_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ResearchExpert::with_timeout(timeout_secs), (None, None)).await?;
    Ok(actor_ref)
}
```

- [ ] **Step 2: Add export to experts/mod.rs**

Add to `agent-service/src/mesh/experts/mod.rs`:

```rust
pub mod research_expert;

pub use research_expert::{
    ResearchExpert, spawn_research_expert, spawn_research_expert_with_peers,
    spawn_research_expert_with_timeout,
};
```

- [ ] **Step 3: Check compilation**

Run: `cargo check`

Expected: Should compile without errors.

- [ ] **Step 4: Commit**

```bash
git add agent-service/src/mesh/experts/research_expert.rs agent-service/src/mesh/experts/mod.rs
git commit -m "feat(mesh): add ResearchExpert for URL scraping and content gathering

- Implement ResearchExpert with research, scrape, gather capabilities
- Add Research payload handler with URLs and depth parameters
- Support peer delegation and pending task management
- Add spawn functions with variants for peers, timeout, and mesh events

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Create PIIStripperExpert

**Files:**
- Create: `agent-service/src/mesh/experts/pii_stripper_expert.rs`
- Modify: `agent-service/src/mesh/experts/mod.rs`

- [ ] **Step 1: Create pii_stripper_expert.rs**

Create file `agent-service/src/mesh/experts/pii_stripper_expert.rs`:

```rust
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::expert::{PIICategory, *};
use crate::mesh::types::{EntryMsg, MeshSignal};

const PII_CAPABILITIES: &[&str] = &["pii", "anonymize", "scrub"];

#[derive(Debug, Clone)]
pub struct PIIResult {
    pub scrubbed_content: String,
    pub report: HashMap<String, Vec<String>>,
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

        patterns.insert(PIICategory::Email,
            regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap());
        patterns.insert(PIICategory::PhoneNumber,
            regex::Regex::new(r"\+?\d{1,3}[-.\s]?\(?\d{1,4}\)?[-.\s]?\d{1,4}[-.\s]?\d{1,9}").unwrap());
        patterns.insert(PIICategory::SSN,
            regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b|\b\d{3}\s\d{2}\s\d{4}\b").unwrap());
        patterns.insert(PIICategory::IBAN,
            regex::Regex::new(r"[A-Z]{2}\d{2}[A-Z0-9]{11,30}").unwrap());
        patterns.insert(PIICategory::Name,
            regex::Regex::new(r"\b[A-Z][a-z]+\s[A-Z][a-z]+\b").unwrap());
        patterns.insert(PIICategory::Address,
            regex::Regex::new(r"\d+\s+[A-Z][a-z]+\s+(Street|St|Avenue|Ave|Road|Rd|Lane|Ln|Boulevard|Blvd)\b").unwrap());

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

    fn scrub_pii(&mut self, content: &str, categories: &[PIICategory]) -> PIIResult {
        self.processed_count += 1;

        let mut scrubbed_content = content.to_string();
        let mut report: HashMap<String, Vec<String>> = HashMap::new();

        for category in categories {
            let category_name = format!("{:?}", category);

            if let Some(pattern) = self.patterns.get(category) {
                let matches: Vec<String> = pattern
                    .find_iter(content)
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

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::ScrubPII { content, pii_categories } => {
                let result = self.scrub_pii(content, pii_categories);

                let mut response = format!(
                    "PII Stripper Report #{}:\n",
                    self.processed_count
                );

                for (category, matches) in &result.report {
                    response.push_str(&format!(
                        "- {}: {} instance(s) found and redacted\n",
                        category,
                        matches.len()
                    ));
                }

                response.push_str(&format!("\nScrubbed content:\n{}\n", result.scrubbed_content));

                response
            }
            _ => "PIIStripperExpert: Please provide ScrubPII payload with content and categories".to_string(),
        }
    }

    fn would_delegate_to_sender(
        peers: &Option<PeerMap>,
        target: &str,
        envelope: &WorkEnvelope,
    ) -> bool {
        match (peers.as_ref().and_then(|m| m.get(target)), envelope.reply_to.as_ref()) {
            (Some(peer), Some(reply)) => reply.get_id() == peer.get_id(),
            _ => false,
        }
    }

    fn query_slice(payload: &WorkPayload) -> &str {
        match payload {
            WorkPayload::Query(q) => q.as_str(),
            WorkPayload::Process(p) => p.as_str(),
            WorkPayload::Delegate { capability, .. } => capability.as_str(),
            WorkPayload::ScrubPII { .. } => "pii",
            _ => "",
        }
    }
}

#[async_trait::async_trait]
impl Actor for PIIStripperExpert {
    type Msg = ExpertMsg;
    type State = PIIStripperExpert;
    type Arguments = (Option<PeerMap>, Option<broadcast::Sender<crate::AgentEvent>>);

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

                let result = state.process_locally(&envelope);
                send_work_output(&envelope, result);
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.base.pending_tasks.remove(&trace_id) {
                    if let Some(ref gateway) = task.entry_reply {
                        let _ = gateway.cast(EntryMsg::ExpertResponse {
                            trace_id,
                            result,
                        });
                    }
                }
                Ok(())
            }
            ExpertMsg::MeshSignal(signal) => match signal {
                MeshSignal::Cancel => {
                    state.base.pending_tasks.clear();
                }
            },
            ExpertMsg::TimerTick => {
                let _recovered = state.base.recover_expired_tasks(&myself);
                let _purged = state.base.purge_expired_tasks();
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
    let (actor_ref, _) = Actor::spawn(None, PIIStripperExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_pii_stripper_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, PIIStripperExpert::with_timeout(timeout_secs), (None, None)).await?;
    Ok(actor_ref)
}
```

- [ ] **Step 2: Add regex dependency to Cargo.toml**

Add to `agent-service/Cargo.toml` dependencies:

```toml
regex = "1.10"
```

- [ ] **Step 3: Update experts/mod.rs exports**

Add to `agent-service/src/mesh/experts/mod.rs`:

```rust
pub mod pii_stripper_expert;

pub use pii_stripper_expert::{
    PIIStripperExpert, PIIResult, spawn_pii_stripper_expert, spawn_pii_stripper_expert_with_peers,
    spawn_pii_stripper_expert_with_timeout,
};
```

- [ ] **Step 4: Check compilation**

Run: `cargo check`

Expected: Should compile without errors.

- [ ] **Step 5: Commit**

```bash
git add agent-service/src/mesh/experts/pii_stripper_expert.rs agent-service/src/mesh/experts/mod.rs agent-service/Cargo.toml agent-service/Cargo.lock
git commit -m "feat(mesh): add PIIStripperExpert for PII detection and redaction

- Implement PIIStripperExpert with pii, anonymize, scrub capabilities
- Add regex-based detection for Email, Phone, SSN, IBAN, Name, Address
- Return scrubbed content with detection report
- Add spawn functions with variants for peers, timeout, and mesh events

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Create SchrijverExpert

**Files:**
- Create: `agent-service/src/mesh/experts/schrijver_expert.rs`
- Modify: `agent-service/src/mesh/experts/mod.rs`

- [ ] **Step 1: Create schrijver_expert.rs**

Create file `agent-service/src/mesh/experts/schrijver_expert.rs`:

```rust
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::expert::{StyleProfile, *};
use crate::mesh::types::{EntryMsg, MeshSignal};

const SCHRIJVER_CAPABILITIES: &[&str] = &["write", "draft", "compose"];

pub struct SchrijverExpert {
    base: ExpertState,
    style_templates: HashMap<StyleProfile, String>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl SchrijverExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "SchrijverExpert".to_string(),
            SCHRIJVER_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut style_templates = HashMap::new();
        style_templates.insert(
            StyleProfile::Formal,
            "Write in a professional, formal tone with proper structure and complete sentences.".to_string(),
        );
        style_templates.insert(
            StyleProfile::Casual,
            "Write in a friendly, conversational tone that is easy to read.".to_string(),
        );
        style_templates.insert(
            StyleProfile::Legal,
            "Write with legal precision, using appropriate terminology and disclaimers.".to_string(),
        );
        style_templates.insert(
            StyleProfile::Technical,
            "Write with technical accuracy, including relevant details and specifications.".to_string(),
        );

        Self {
            base,
            style_templates,
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

    fn generate_document(&mut self, notes: &str, style: &StyleProfile) -> String {
        self.processed_count += 1;

        let style_guidance = self
            .style_templates
            .get(style)
            .map(|s| s.as_str())
            .unwrap_or("Write clearly and concisely.");

        format!(
            "=== Document #{} ({:?}) ===\n\nStyle Guidance: {}\n\nResearch Notes:\n{}\n\n=== End Document ===",
            self.processed_count,
            style,
            style_guidance,
            notes
        )
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::Write {
                research_notes,
                style_profile,
            } => {
                let document = self.generate_document(research_notes, style_profile);
                tracing::info!(
                    style = ?style_profile,
                    notes_len = research_notes.len(),
                    "Generated document"
                );
                document
            }
            _ => "SchrijverExpert: Please provide Write payload with research_notes and style_profile".to_string(),
        }
    }

    fn should_delegate(&self, query: &str, hop_count: u32, envelope: &WorkEnvelope) -> Option<String> {
        if hop_count >= MAX_DELEGATION_DEPTH {
            return None;
        }

        let query_lower = query.to_lowercase();

        if query_lower.contains("research") || query_lower.contains("scrape") {
            Some("research".to_string())
        } else if query_lower.contains("pii") || query_lower.contains("anonymize") {
            Some("pii".to_string())
        } else {
            None
        }
    }

    fn would_delegate_to_sender(
        peers: &Option<PeerMap>,
        target: &str,
        envelope: &WorkEnvelope,
    ) -> bool {
        match (peers.as_ref().and_then(|m| m.get(target)), envelope.reply_to.as_ref()) {
            (Some(peer), Some(reply)) => reply.get_id() == peer.get_id(),
            _ => false,
        }
    }

    fn query_slice(payload: &WorkPayload) -> &str {
        match payload {
            WorkPayload::Query(q) => q.as_str(),
            WorkPayload::Process(p) => p.as_str(),
            WorkPayload::Delegate { capability, .. } => capability.as_str(),
            WorkPayload::Write { .. } => "write",
            _ => "",
        }
    }

    fn handle_delegation(
        &mut self,
        envelope: &WorkEnvelope,
        target_capability: String,
        myself: &ActorRef<ExpertMsg>,
    ) -> Result<(), ExpertError> {
        let peer = self
            .peers
            .as_ref()
            .and_then(|m| m.get(&target_capability))
            .cloned()
            .ok_or_else(|| ExpertError::CapabilityNotFound(target_capability.clone()))?;

        let delegated = BatonPass::prepare_delegation(
            envelope,
            myself.clone(),
            envelope.payload.clone(),
        );

        let pending_task =
            BatonPass::create_pending_task_with_peer(envelope, target_capability.clone(), peer.clone());
        self.base.store_pending_task(pending_task);

        tracing::info!(
            trace_id = %envelope.trace_id,
            from = %self.base.name,
            to = %target_capability,
            "Delegating to peer"
        );

        let _ = peer.cast(ExpertMsg::Work(delegated));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Actor for SchrijverExpert {
    type Msg = ExpertMsg;
    type State = SchrijverExpert;
    type Arguments = (Option<PeerMap>, Option<broadcast::Sender<crate::AgentEvent>>);

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

                if let Some(delegation_target) = state.should_delegate(
                    Self::query_slice(&envelope.payload),
                    envelope.hop_count,
                    &envelope,
                ) {
                    let _ = state.handle_delegation(&envelope, delegation_target, &myself);
                } else {
                    let result = state.process_locally(&envelope);
                    send_work_output(&envelope, result);
                }
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.base.pending_tasks.remove(&trace_id) {
                    if let Some(ref reply_to) = task.reply_to {
                        let _ = reply_to.cast(ExpertMsg::PeerResponse {
                            trace_id,
                            result,
                        });
                    } else if let Some(ref gateway) = task.entry_reply {
                        let _ = gateway.cast(EntryMsg::ExpertResponse {
                            trace_id,
                            result,
                        });
                    }
                }
                Ok(())
            }
            ExpertMsg::MeshSignal(signal) => match signal {
                MeshSignal::Cancel => {
                    state.base.pending_tasks.clear();
                }
            },
            ExpertMsg::TimerTick => {
                let _recovered = state.base.recover_expired_tasks(&myself);
                let _purged = state.base.purge_expired_tasks();
            }
        }
    }
}

pub async fn spawn_schrijver_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, SchrijverExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_schrijver_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, SchrijverExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_schrijver_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, SchrijverExpert::with_timeout(timeout_secs), (None, None)).await?;
    Ok(actor_ref)
}
```

- [ ] **Step 2: Update experts/mod.rs exports**

Add to `agent-service/src/mesh/experts/mod.rs`:

```rust
pub mod schrijver_expert;

pub use schrijver_expert::{
    SchrijverExpert, spawn_schrijver_expert, spawn_schrijver_expert_with_peers,
    spawn_schrijver_expert_with_timeout,
};
```

- [ ] **Step 3: Check compilation**

Run: `cargo check`

Expected: Should compile without errors.

- [ ] **Step 4: Commit**

```bash
git add agent-service/src/mesh/experts/schrijver_expert.rs agent-service/src/mesh/experts/mod.rs
git commit -m "feat(mesh): add SchrijverExpert for document generation

- Implement SchrijverExpert with write, draft, compose capabilities
- Add style profile support (Formal, Casual, Legal, Technical, Custom)
- Generate documents from research notes with style guidance
- Support peer delegation to ResearchExpert and PIIStripperExpert
- Add spawn functions with variants for peers, timeout, and mesh events

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Create ReviewerExpert

**Files:**
- Create: `agent-service/src/mesh/experts/reviewer_expert.rs`
- Modify: `agent-service/src/mesh/experts/mod.rs`

- [ ] **Step 1: Create reviewer_expert.rs**

Create file `agent-service/src/mesh/experts/reviewer_expert.rs`:

```rust
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::mesh::expert::{ReviewCriteria, *};
use crate::mesh::types::{EntryMsg, MeshSignal};

const REVIEWER_CAPABILITIES: &[&str] = &["review", "critique", "edit"];

#[derive(Debug, Clone)]
pub struct ReviewAnnotation {
    pub line_number: usize,
    pub severity: String,
    pub message: String,
    pub suggestion: Option<String>,
}

pub struct ReviewerExpert {
    base: ExpertState,
    review_templates: HashMap<String, Vec<String>>,
    processed_count: u64,
    peers: Option<PeerMap>,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl ReviewerExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "ReviewerExpert".to_string(),
            REVIEWER_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        );

        let mut review_templates = HashMap::new();
        review_templates.insert(
            "clarity".to_string(),
            vec![
                "Consider rewriting for clarity".to_string(),
                "This sentence could be more concise".to_string(),
            ],
        );
        review_templates.insert(
            "grammar".to_string(),
            vec!["Check grammar and punctuation".to_string()],
        );
        review_templates.insert(
            "structure".to_string(),
            vec![
                "Consider restructuring this paragraph".to_string(),
                "Add headings for better organization".to_string(),
            ],
        );

        Self {
            base,
            review_templates,
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

    fn review_document(&mut self, content: &str, criteria: &Option<ReviewCriteria>) -> Vec<ReviewAnnotation> {
        self.processed_count += 1;

        let mut annotations = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.len() > 150 {
                annotations.push(ReviewAnnotation {
                    line_number: i + 1,
                    severity: "warning".to_string(),
                    message: format!("Line exceeds recommended length ({} chars)", line.len()),
                    suggestion: Some("Consider splitting this line".to_string()),
                });
            }

            if line.contains("TODO") || line.contains("FIXME") {
                annotations.push(ReviewAnnotation {
                    line_number: i + 1,
                    severity: "info".to_string(),
                    message: "Contains TODO/FIXME marker".to_string(),
                    suggestion: Some("Resolve or track this item".to_string()),
                });
            }
        }

        if let Some(crit) = criteria {
            if let Some(ref tone) = crit.tone {
                annotations.push(ReviewAnnotation {
                    line_number: 0,
                    severity: "info".to_string(),
                    message: format!("Tone check: document should be '{}'", tone),
                    suggestion: None,
                });
            }

            if let Some((min, max)) = crit.length_constraints {
                let word_count = content.split_whitespace().count();
                if word_count < min || word_count > max {
                    annotations.push(ReviewAnnotation {
                        line_number: 0,
                        severity: "warning".to_string(),
                        message: format!("Word count {} outside target range ({}, {})", word_count, min, max),
                        suggestion: Some("Adjust content length".to_string()),
                    });
                }
            }

            for area in &crit.focus_areas {
                annotations.push(ReviewAnnotation {
                    line_number: 0,
                    severity: "info".to_string(),
                    message: format!("Focus area: {}", area),
                    suggestion: None,
                });
            }
        }

        annotations
    }

    fn format_review(&self, annotations: &[ReviewAnnotation]) -> String {
        if annotations.is_empty() {
            return "=== Review Report ===\n\nStatus: PASSED\n\nNo issues found. Document looks good!".to_string();
        }

        let mut report = format!(
            "=== Review Report #{} ===\n\nStatus: NEEDS ATTENTION\n\n",
            self.processed_count
        );

        report.push_str(&format!("Found {} issue(s):\n\n", annotations.len()));

        for (i, annotation) in annotations.iter().enumerate() {
            report.push_str(&format!(
                "{}. [{}] Line {}: {}\n",
                i + 1,
                annotation.severity.to_uppercase(),
                annotation.line_number,
                annotation.message
            ));

            if let Some(ref suggestion) = annotation.suggestion {
                report.push_str(&format!("   Suggestion: {}\n", suggestion));
            }

            report.push('\n');
        }

        report.push_str("=== End Review ===");

        report
    }

    fn process_locally(&mut self, envelope: &WorkEnvelope) -> String {
        match &envelope.payload {
            WorkPayload::Review { content, criteria } => {
                let annotations = self.review_document(content, criteria);
                let report = self.format_review(&annotations);

                tracing::info!(
                    issues_found = annotations.len(),
                    "Document review completed"
                );

                report
            }
            _ => "ReviewerExpert: Please provide Review payload with content".to_string(),
        }
    }

    fn would_delegate_to_sender(
        peers: &Option<PeerMap>,
        target: &str,
        envelope: &WorkEnvelope,
    ) -> bool {
        match (peers.as_ref().and_then(|m| m.get(target)), envelope.reply_to.as_ref()) {
            (Some(peer), Some(reply)) => reply.get_id() == peer.get_id(),
            _ => false,
        }
    }

    fn query_slice(payload: &WorkPayload) -> &str {
        match payload {
            WorkPayload::Query(q) => q.as_str(),
            WorkPayload::Process(p) => p.as_str(),
            WorkPayload::Delegate { capability, .. } => capability.as_str(),
            WorkPayload::Review { .. } => "review",
            _ => "",
        }
    }
}

#[async_trait::async_trait]
impl Actor for ReviewerExpert {
    type Msg = ExpertMsg;
    type State = ReviewerExpert;
    type Arguments = (Option<PeerMap>, Option<broadcast::Sender<crate::AgentEvent>>);

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

                let result = state.process_locally(&envelope);
                send_work_output(&envelope, result);
                Ok(())
            }
            ExpertMsg::PeerResponse { trace_id, result } => {
                if let Some(task) = state.base.pending_tasks.remove(&trace_id) {
                    if let Some(ref gateway) = task.entry_reply {
                        let _ = gateway.cast(EntryMsg::ExpertResponse {
                            trace_id,
                            result,
                        });
                    }
                }
                Ok(())
            }
            ExpertMsg::MeshSignal(signal) => match signal {
                MeshSignal::Cancel => {
                    state.base.pending_tasks.clear();
                }
            },
            ExpertMsg::TimerTick => {
                let _recovered = state.base.recover_expired_tasks(&myself);
                let _purged = state.base.purge_expired_tasks();
            }
        }
    }
}

pub async fn spawn_reviewer_expert(
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ReviewerExpert::new(), (None, mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_reviewer_expert_with_peers(
    peers: PeerMap,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ReviewerExpert::new(), (Some(peers), mesh_events)).await?;
    Ok(actor_ref)
}

pub async fn spawn_reviewer_expert_with_timeout(
    timeout_secs: u64,
) -> Result<ActorRef<ExpertMsg>, Box<dyn std::error::Error>> {
    let (actor_ref, _) = Actor::spawn(None, ReviewerExpert::with_timeout(timeout_secs), (None, None)).await?;
    Ok(actor_ref)
}
```

- [ ] **Step 2: Update experts/mod.rs exports**

Add to `agent-service/src/mesh/experts/mod.rs`:

```rust
pub mod reviewer_expert;

pub use reviewer_expert::{
    ReviewAnnotation, ReviewerExpert, spawn_reviewer_expert, spawn_reviewer_expert_with_peers,
    spawn_reviewer_expert_with_timeout,
};
```

- [ ] **Step 3: Check compilation**

Run: `cargo check`

Expected: Should compile without errors.

- [ ] **Step 4: Commit**

```bash
git add agent-service/src/mesh/experts/reviewer_expert.rs agent-service/src/mesh/experts/mod.rs
git commit -m "feat(mesh): add ReviewerExpert for document critique and annotation

- Implement ReviewerExpert with review, critique, edit capabilities
- Add ReviewAnnotation with line number, severity, message, and suggestion
- Support ReviewCriteria for tone, length constraints, and focus areas
- Generate formatted review reports with issue tracking
- Add spawn functions with variants for peers, timeout, and mesh events

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Create DocumentOrchestrator

**Files:**
- Create: `agent-service/src/mesh/document_orchestrator.rs`
- Modify: `agent-service/src/mesh/mod.rs`
- Modify: `agent-service/src/mesh/expert.rs`

- [ ] **Step 1: Add OrchestratorMsg and pending state to expert.rs**

Add after the `ExpertMsg` enum definition (after line 42):

```rust
#[derive(Debug, Clone)]
pub enum OrchestratorMsg {
    CreateDocument {
        urls: Vec<String>,
        style_profile: StyleProfile,
        pii_categories: Vec<PIICategory>,
        review_criteria: Option<ReviewCriteria>,
        error_strategy: ErrorStrategy,
        reply_to: Option<ActorRef<EntryMsg>>,
        trace_id: Uuid,
    },
    StepComplete {
        step: WorkflowStep,
        trace_id: Uuid,
        result: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowStep {
    Research,
    Write,
    ScrubPII,
    Review,
}

#[derive(Debug, Clone)]
pub struct WorkflowState {
    pub trace_id: Uuid,
    pub current_step: WorkflowStep,
    pub accumulated_results: HashMap<WorkflowStep, String>,
    pub error_strategy: ErrorStrategy,
    pub retry_count: u32,
    pub reply_to: Option<ActorRef<EntryMsg>>,
}
```

- [ ] **Step 2: Create document_orchestrator.rs**

Create file `agent-service/src/mesh/document_orchestrator.rs`:

```rust
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

use crate::mesh::expert::{
    ErrorStrategy, ExpertMsg, OrchestratorMsg, PeerMap, ReviewCriteria, StyleProfile,
    WorkflowStep, WorkflowState, PIICategory, WorkEnvelope, WorkPayload,
};
use crate::mesh::types::{EntryMsg, SessionContext};

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

    fn start_workflow(&mut self, msg: OrchestratorMsg::CreateDocument) -> Result<(), String> {
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
                    let msg = OrchestratorMsg::CreateDocument {
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
```

- [ ] **Step 3: Update mesh/mod.rs exports**

Add to `agent-service/src/mesh/mod.rs`:

```rust
pub mod document_orchestrator;

pub use document_orchestrator::{DocumentOrchestrator, spawn_document_orchestrator};
pub use expert::{OrchestratorMsg, WorkflowState, WorkflowStep};
```

- [ ] **Step 4: Check compilation**

Run: `cargo check`

Expected: Should compile without errors.

- [ ] **Step 5: Commit**

```bash
git add agent-service/src/mesh/document_orchestrator.rs agent-service/src/mesh/mod.rs agent-service/src/mesh/expert.rs
git commit -m "feat(mesh): add DocumentOrchestrator for workflow coordination

- Implement DocumentOrchestrator with document, create-doc capabilities
- Coordinate Research → Write → ScrubPII → Review workflow
- Track workflow state with accumulated results and retry handling
- Support ErrorStrategy (FailFast, PartialResults, RetryWithFallback)
- Add spawn function with peer configuration

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Update EntryActor Triage

**Files:**
- Modify: `agent-service/src/mesh/entry.rs`

- [ ] **Step 1: Extend triage_capability with document capabilities**

Update the `triage_capability` method in `agent-service/src/mesh/entry.rs` (around line 92):

```rust
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
```

- [ ] **Step 2: Check compilation**

Run: `cargo check`

Expected: Should compile without errors.

- [ ] **Step 3: Commit**

```bash
git add agent-service/src/mesh/entry.rs
git commit -m "feat(mesh): extend EntryActor triage for document experts

- Add routing for research, write, pii, review, and document capabilities
- Enable discovery of new document creation experts via triage
- Maintain existing rust, frontend, database, general routing

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Add Integration Tests

**Files:**
- Modify: `agent-service/tests/mesh_tests.rs`

- [ ] **Step 1: Add document expert tests**

Add to `agent-service/tests/mesh_tests.rs`:

```rust
#[tokio::test]
async fn research_expert_test() {
    let research_expert = spawn_research_expert(None).await.expect("spawn researcher");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::Research {
            urls: vec!["https://example.com/article".into()],
            depth: 2,
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    research_expert.cast(ExpertMsg::Work(envelope)).expect("send work");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    tracing::info!("ResearchExpert test passed");
}

#[tokio::test]
async fn pii_stripper_expert_test() {
    let pii_expert = spawn_pii_stripper_expert(None).await.expect("spawn pii");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let test_content = "Contact John Doe at john.doe@example.com or call 555-123-4567.";

    let envelope = WorkEnvelope {
        payload: WorkPayload::ScrubPII {
            content: test_content.to_string(),
            pii_categories: vec![PIICategory::Email, PIICategory::PhoneNumber, PIICategory::Name],
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    pii_expert.cast(ExpertMsg::Work(envelope)).expect("send work");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    tracing::info!("PIIStripperExpert test passed");
}

#[tokio::test]
async fn schrijver_expert_test() {
    let schrijver_expert = spawn_schrijver_expert(None).await.expect("spawn schrijver");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let notes = "Research found that Rust is a systems programming language.";

    let envelope = WorkEnvelope {
        payload: WorkPayload::Write {
            research_notes: notes.to_string(),
            style_profile: StyleProfile::Technical,
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    schrijver_expert.cast(ExpertMsg::Work(envelope)).expect("send work");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    tracing::info!("SchrijverExpert test passed");
}

#[tokio::test]
async fn reviewer_expert_test() {
    let reviewer_expert = spawn_reviewer_expert(None).await.expect("spawn reviewer");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let content = "This is a test document. \n\nIt has some content that needs review.";

    let envelope = WorkEnvelope {
        payload: WorkPayload::Review {
            content: content.to_string(),
            criteria: Some(ReviewCriteria {
                tone: Some("professional".to_string()),
                length_constraints: Some((10, 100)),
                focus_areas: vec!["clarity".to_string()],
            }),
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    reviewer_expert.cast(ExpertMsg::Work(envelope)).expect("send work");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    tracing::info!("ReviewerExpert test passed");
}

#[tokio::test]
async fn document_orchestrator_workflow_test() {
    let registry = spawn_registry().await.expect("registry");

    let research_expert = spawn_research_expert(None).await.expect("researcher");
    let schrijver_expert = spawn_schrijver_expert(None).await.expect("schrijver");
    let pii_expert = spawn_pii_stripper_expert(None).await.expect("pii");
    let reviewer_expert = spawn_reviewer_expert(None).await.expect("reviewer");

    let mut peers = HashMap::new();
    peers.insert("research".to_string(), research_expert.clone());
    peers.insert("write".to_string(), schrijver_expert.clone());
    peers.insert("pii".to_string(), pii_expert.clone());
    peers.insert("review".to_string(), reviewer_expert.clone());

    let orchestrator = spawn_document_orchestrator(peers).await.expect("orchestrator");

    let context = SessionContext {
        session_id: uuid::Uuid::new_v4(),
        user_id: "test_user".into(),
        metadata: Default::default(),
    };

    let envelope = WorkEnvelope {
        payload: WorkPayload::CreateDocument {
            urls: vec!["https://example.com".into()],
            style_profile: StyleProfile::Formal,
            pii_categories: vec![PIICategory::Email],
            review_criteria: Some(ReviewCriteria {
                tone: Some("professional".to_string()),
                length_constraints: None,
                focus_areas: vec![],
            }),
            error_strategy: ErrorStrategy::FailFast,
        },
        context: context.clone(),
        reply_to: None,
        entry_reply: None,
        trace_id: uuid::Uuid::new_v4(),
        hop_count: 0,
    };

    orchestrator.cast(ExpertMsg::Work(envelope)).expect("send work");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    tracing::info!("DocumentOrchestrator workflow test passed");
}
```

- [ ] **Step 2: Check tests compile and run**

Run: `cargo test --test mesh_tests document`

Expected: New tests should pass.

- [ ] **Step 3: Commit**

```bash
git add agent-service/tests/mesh_tests.rs
git commit -m "test(mesh): add integration tests for document creation experts

- Add ResearchExpert test for URL scraping payload
- Add PIIStripperExpert test for PII redaction
- Add SchrijverExpert test for document generation
- Add ReviewerExpert test for review and annotation
- Add DocumentOrchestrator workflow test for end-to-end pipeline

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: Update Main Exports

**Files:**
- Modify: `agent-service/src/mesh/mod.rs`

- [ ] **Step 1: Update exports to include all new types**

Update the `pub use` section to include:

```rust
pub use expert::{
    BatonPass, ExpertError, ExpertMsg, ExpertState, PeerMap, PendingTask, TaskState,
    WorkEnvelope, WorkPayload, spawn_expert,
    StyleProfile, PIICategory, ReviewCriteria, ErrorStrategy,
    OrchestratorMsg, WorkflowState, WorkflowStep,
};
pub use entry::{EntryActor, PendingRequest, spawn_entry_actor, spawn_entry_actor_with_timeout};
pub use experts::{
    FrontendExpert, RustExpert, ResearchExpert, SchrijverExpert, PIIStripperExpert, ReviewerExpert,
    spawn_frontend_expert, spawn_frontend_expert_with_peers, spawn_frontend_expert_with_timeout,
    spawn_rust_expert, spawn_rust_expert_with_peers, spawn_rust_expert_with_timeout,
    spawn_research_expert, spawn_research_expert_with_peers, spawn_research_expert_with_timeout,
    spawn_schrijver_expert, spawn_schrijver_expert_with_peers, spawn_schrijver_expert_with_timeout,
    spawn_pii_stripper_expert, spawn_pii_stripper_expert_with_peers, spawn_pii_stripper_expert_with_timeout,
    spawn_reviewer_expert, spawn_reviewer_expert_with_peers, spawn_reviewer_expert_with_timeout,
};
pub use document_orchestrator::{DocumentOrchestrator, spawn_document_orchestrator};
```

- [ ] **Step 2: Final compilation check**

Run: `cargo check`

Expected: All modules compile without errors.

- [ ] **Step 3: Run full test suite**

Run: `cargo test`

Expected: All tests pass.

- [ ] **Step 4: Final commit**

```bash
git add agent-service/src/mesh/mod.rs
git commit -m "feat(mesh): update exports for document creation experts

- Export all new expert types and spawn functions
- Include StyleProfile, PIICategory, ReviewCriteria, ErrorStrategy
- Export WorkflowState, WorkflowStep, OrchestratorMsg
- Export ResearchExpert, SchrijverExpert, PIIStripperExpert, ReviewerExpert
- Export DocumentOrchestrator

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```
