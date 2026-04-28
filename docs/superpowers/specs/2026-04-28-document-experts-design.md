# Document Creation Expert Agents - Design Spec

**Date:** 2026-04-28
**Status:** Approved
**Author:** Claude + Marc

## Overview

Uitbreiding van de bestaande actor mesh met expert agents voor document creatie: Researcher, Schrijver, PII-Stripper, en Reviewer. Een `DocumentOrchestrator` coördineert de volledige workflow met configureerbare error handling.

## Architecture

```
                    ┌─────────────────────┐
                    │    EntryActor       │
                    │  (triage capability)│
                    │   "document" etc.   │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │ DocumentOrchestrator │
                    │  (sequences workflow) │
                    │  error_strategy: D   │
                    └──────────┬───────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         │                     │                     │
         ▼                     ▼                     ▼
   ┌─────────┐           ┌─────────┐           ┌─────────┐
   │Research │    ┌────► │ Schrijver│   ┌────►│   PII    │
   │ Expert  │    │      │ Expert   │   │     │ Stripper │
   └─────────┘    │      └─────────┘   │     └─────────┘
                 │                     │
                 │     ┌──────────┐    │
                 └────►│ Reviewer │◄───┘
                       │  Expert  │
                       └──────────┘
```

## Experts

| Expert | Capabilities | Input | Output | Peer Dependencies |
|--------|--------------|-------|--------|-------------------|
| `ResearchExpert` | `["research", "scrape", "gather"]` | URLs | Research notities | Geen |
| `SchrijverExpert` | `["write", "draft", "compose"]` | Notities + stijl | Concept tekst | ResearchExpert |
| `PIIStripperExpert` | `["pii", "anonymize", "scrub"]` | Tekst + categorieën | Geanonimiseerd + rapport | Geen |
| `ReviewerExpert` | `["review", "critique", "edit"]` | Tekst + criteria | Annotaties + suggesties | Geen |
| `DocumentOrchestrator` | `["document", "create-doc"]` | Volledige request | Final document | Alle 4 experts |

## Data Structures

### WorkPayload Extensions

```rust
pub enum WorkPayload {
    // Bestaande...
    Query(String),
    Process(String),
    Delegate { capability: String, payload: String },

    // Nieuw
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

### Supporting Types

```rust
#[derive(Debug, Clone)]
pub enum StyleProfile {
    Formal,
    Casual,
    Legal,
    Technical,
    Custom(String),
}

#[derive(Debug, Clone)]
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

## Workflow

**DocumentOrchestrator flow:**

1. Ontvangt `CreateDocument` payload
2. Stuurt `Research` naar ResearchExpert (met error_strategy)
3. Bij succes: resultaat → SchrijverExpert
4. Bij fout (retry mode): probeer alternatieven, dan fallback
5. Schrijver → concept → PII-Stripper
6. PII-Stripper → schoon → Reviewer
7. Eindresultaat via `entry_reply` terug naar gebruiker

```
Research → [notes] → Write → [draft] → ScrubPII → [clean] → Review → [final]
     ↓retry              ↓retry           ↓retry            ↓retry
```

## File Structure

```
agent-service/src/mesh/
├── document_orchestrator.rs   # NIEUW - workflow coördinator
├── entry.rs                    # UPDATE - triage uitbreiden
└── expert.rs                   # UPDATE - WorkPayload uitbreiden

agent-service/src/mesh/experts/
├── mod.rs                      # UPDATE - exports
├── research_expert.rs          # NIEUW
├── schrijver_expert.rs         # NIEUW
├── pii_stripper_expert.rs      # NIEUW
└── reviewer_expert.rs          # NIEUW
```

## EntryActor Triage Updates

```rust
fn triage_capability(&self, query: &str) -> String {
    let query_lower = query.to_lowercase();

    if query_lower.contains("research") || query_lower.contains("scrape") {
        "research".to_string()
    } else if query_lower.contains("write") || query_lower.contains("draft") {
        "write".to_string()
    } else if query_lower.contains("pii") || query_lower.contains("anonymize") {
        "pii".to_string()
    } else if query_lower.contains("review") || query_lower.contains("critique") {
        "review".to_string()
    } else if query_lower.contains("document") || query_lower.contains("create doc") {
        "document".to_string()
    }
    // ... bestaande rust/frontend/database ...
}
```

## Implementation Order

**Fase 1 - Basis experts (onafhankelijk):**
1. `ResearchExpert` - URL scraping logic
2. `PIIStripperExpert` - regex-based PII detection

**Fase 2 - Creatie experts:**
3. `SchrijverExpert` - research → tekst generatie
4. `ReviewerExpert` - tekst → annotaties

**Fase 3 - Coördinatie:**
5. `DocumentOrchestrator` - workflow orkestratie
6. EntryActor triage uitbreiden
7. Integration tests

## Error Handling

Elke expert ondersteunt de geselecteerde `ErrorStrategy`:

- **FailFast:** Eerste fout stopt de workflow
- **PartialResults:** Geeft terug wat er is, markeert fouten
- **RetryWithFallback:** Probeert opnieuw met alternatieven

De DocumentOrchestrator geeft de strategy door aan elke sub-expert en handelt tussenresultaten correct af.
