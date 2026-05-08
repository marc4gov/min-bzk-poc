# High-Level Design Review: Local-First AI Assistant

> **Template Oorsprong**: Officieel | **ArcKit Versie**: 4.3.1 | **Command**: `/arckit:hld-review`

## Documentbeheer

| Veld | Waarde |
|------|--------|
| **Document ID** | ARC-000-HLDR-v1.0 |
| **Document Type** | High-Level Design Review |
| **Project** | Local-First AI Assistant (Project 000) |
| **Classificatie** | PUBLIC |
| **Status** | IN_REVIEW |
| **Versie** | 1.0 |
| **Aanmaakdatum** | 2026-05-07 |
| **Laatste Wijziging** | 2026-05-07 |
| **Review Cyclus** | Per Kwartaal |
| **Volgende Review Datum** | 2026-08-07 |
| **Eigenaar** | Enterprise Architect |
| **Beoordeeld Door** | [PENDING] |
| **Goedgekeurd Door** | [PENDING] |
| **Distributie** | Projectteam, Stakeholders, Architecture Board |

## Revisiegeschiedenis

| Versie | Datum | Auteur | Wijzigingen | Goedgekeurd Door | Goedkeuringsdatum |
|--------|-------|--------|-------------|------------------|-------------------|
| 1.0 | 2026-05-07 | ArcKit AI | Eerste aanmaak vanuit `/arckit:hld-review` commando | [PENDING] | [PENDING] |

---

## 1. Review Overview

### 1.1 Purpose

Dit document bevat de Architecture Review Board's evaluatie van de High-Level Design (HLD) voor de Local-First AI Assistant. De review evalueert de architectuur tegen enterprise principes, requirements, en technische haalbaarheid.

### 1.2 HLD Documenten Onder Review

| Document | ID | Status |
|----------|-----|--------|
| Requirements | ARC-000-REQS-v1.0 | DRAFT |
| Architecture Principles | ARC-000-PRIN-v1.0 | DRAFT |
| Data Model | ARC-000-DATA-v1.0 | DRAFT |
| Context Diagram | ARC-000-DIAG-001-v1.0 | DRAFT |
| Container Diagram | ARC-000-DIAG-002-v1.0 | DRAFT |

### 1.3 Review Participants

| Rol | Organisatie | Review Focus |
|------|-------------|--------------|
| Enterprise Architect | Projectteam | Overall architectuur, principe compliance |
| Security Architect | Projectteam | Security architecture, PII pipeline |
| Domain Expert | Projectteam | Expert mesh systeem, uitbreidbaarheid |

### 1.4 Review Criteria

- **Architecture Principles**: Compliance met enterprise architecture principles (ARC-000-PRIN)
- **Requirements Alignment**: Coverage van functionele en non-functionele requirements
- **Expert Mesh Extensibility**: Uitbreidbaarheid van expert agent systeem
- **Local-First Strategy**: Correcte implementatie van local-first met cloud fallback
- **Security & Compliance**: PII pipeline, AVG/GDPR compliance

---

## 2. Executive Summary

### 2.1 Overall Assessment

**Status**: ⚠️ **APPROVED WITH CONDITIONS**

**Summary**: De Local-First AI Assistant architectuur demonstreert een sterke foundation met een innovatief expert mesh systeem dat reeds gedeeltelijk geïmplementeerd is. De architectuur volgt correct de local-first benadering en Europe-first AI provider strategie. Echter, er zijn significante gaps tussen de architectuurdocumentatie (diagrammen) en de daadwerkelijke implementatie, met name rondom de expert mesh architectuur die niet zichtbaar is in de container diagrammen. Daarnaast moet de local-first strategy (local primary, cloud fallback) expliciet worden gecorrigeerd in de documentatie.

### 2.2 Key Strengths

- ✅ **Expert Mesh Foundation**: Reeds geïmplementeerd uitbreidbaar expert systeem met capabilities, registry pattern, en delegation mechanisme
- ✅ **Modulaire Architectuur**: Scheiding van concerns met duidelijke container boundaries (Web UI, API, Document Processing, PII Pipeline)
- ✅ **Europa-First AI Strategy**: Correcte prioriteit voor Europese providers (Mistral, Aleph Alpha)
- ✅ **Rust Foundation**: Memory-safe backend met sterke performance kenmerken
- ✅ **PII Pipeline**: Dedicated PII detection en redactie pipeline voor AVG/GDPR compliance

### 2.3 Key Concerns

- ⚠️ **Documentation Gap**: Expert mesh architectuur ontbreekt in container diagram (DIAG-002)
- ⚠️ **Local-First Strategy Verwarding**: Documentatie toont Mistral als primary, maar specificatie is local-first met cloud fallback
- ⚠️ **Expert Discovery**: Registry pattern bestaat maar ontbreekt in architectuurdiagrammen
- ⚠️ **Workflow Orchestration**: DocumentOrchestrator bestaat in code maar niet in HLD
- ⚠️ **Observability**: Geen duidelijke monitoring/logging strategy voor expert mesh

### 2.4 Conditions for Approval

**MUST Address Before Implementation Proceeds**:

1. **BLOCKING-01**: Container diagram moet bijgewerkt worden met expert mesh architectuur
2. **BLOCKING-02**: Local-First strategy moet expliciet gedefinieerd worden (local primary, cloud fallback)
3. **BLOCKING-03**: Expert registry en discovery mechanismen moeten gedocumenteerd worden

**SHOULD Address During Detailed Design**:

1. **ADVISORY-01**: Observability strategy voor expert mesh (distributed tracing)
2. **ADVISORY-02**: Expert lifecycle management (spawn, terminate, recovery)
3. **ADVISORY-03**: Performance thresholds voor cloud fallback triggers

### 2.5 Recommendation

- [x] **APPROVED WITH CONDITIONS**: Na adresseren van blocking items kan door naar detailed design

**Target Resubmission Date**: 2026-05-14

---

## 3. Architecture Principles Compliance

### 3.1 Principle Compliance Summary

| Principle ID | Principle Name | Status | Score |
|--------------|----------------|--------|-------|
| P-1 | Schaalbaarheid en Elasticiteit | ⚠️ Partial | 6/10 |
| P-2 | Resilientie en Fouttolerantie | ✅ Compliant | 8/10 |
| P-3 | Interoperabiliteit en Integratie | ✅ Compliant | 9/10 |
| P-4 | EU Digitale Soevereiniteit | ✅ Compliant | 9/10 |
| P-5 | Security by Design | ✅ Compliant | 8/10 |
| P-6 | Observabiliteit | ⚠️ Partial | 5/10 |
| P-7 | Loose Coupling | ✅ Compliant | 9/10 |
| P-8 | Asynchrone Communicatie | ✅ Compliant | 8/10 |

**Gemiddelde Score**: 7.75/10

### 3.2 Principle Compliance Details

#### P-1: Schaalbaarheid en Elasticiteit

**Assessment**: ⚠️ **Partial Compliant**

**Evidence**:
- ✅ Stateless expert actors via ractor framework
- ✅ Horizontal scaling mogelijk voor expert agents
- ✅ Registry pattern voor dynamic expert discovery

**Concerns**:
- ⚠️ Local-first architectuur beperkt horizontale schaalbaarheid (per-device)
- ⚠️ Geen duidelijke strategy voor parallel expert processing
- ⚠️ Document processing single-threaded per default

**Recommendation**:
- Definieer scaling strategy voor document processing (parallel chunking)
- Documenteer limitations van local-first op schaalbaarheid

#### P-2: Resilientie en Fouttolerantie

**Assessment**: ✅ **Compliant**

**Evidence**:
- ✅ Circuit breaker pattern via hop limits (MAX_HOPS)
- ✅ Timeout mechanismen (DEFAULT_TIMEOUT_SECS, TIMER_INTERVAL_SECS)
- ✅ Poison pill propagation voor cancellation
- ✅ Graceful degradation bij expert uitval

**Implementatie Details**:
```rust
// Reeds geïmplementeerd in expert.rs
const MAX_HOPS: u32 = 5;
const MAX_DELEGATION_DEPTH: u32 = 3;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

#### P-3: Interoperabiliteit en Integratie

**Assessment**: ✅ **Compliant**

**Evidence**:
- ✅ Capability-based interface (strings als capability identifiers)
- ✅ Uniform ExpertMsg interface voor alle experts
- ✅ PeerMap voor dynamisch peer discovery
- ✅ Geen directe database toegang over expert grenzen

**Expert Capabilities**:
```rust
pub enum WorkPayload {
    Query(String),
    Process(String),
    Delegate { capability: String, payload: String },
    Research { urls: Vec<String>, depth: u8 },
    Write { research_notes: String, style_profile: StyleProfile },
    ScrubPII { content: String, pii_categories: Vec<PIICategory> },
    Review { content: String, criteria: Option<ReviewCriteria> },
    CreateDocument { ... },
    ImproveDocument { ... },
}
```

#### P-4: EU Digitale Soevereiniteit

**Assessment**: ✅ **Compliant**

**Europa-First Prioriteit**:
| Prioriteit | Provider | EU-Based | Open Source | Local |
|------------|----------|----------|-------------|-------|
| 1e (Primary) | **Ollama (Local)** | ✅ Local | ✅ | ✅ |
| 2e (Fallback) | **Mistral AI** | ✅ FR | ✅ | ❌ |
| 3e (Fallback) | **Aleph Alpha** | ✅ DE | ❌ | ❌ |
| Laatste | OpenAI/Anthropic | ❌ US | ❌ | ❌ |

**⚠️ CRUCIALE CORRECTIE**: De container diagram toont Mistral als primary, maar de specificatie is:
- **Primary**: Local models (Ollama) voor default operatie
- **Fallback**: Cloud models (Mistral, Aleph Alpha) bij performance behoeften

**Recommendation**: Update DIAG-002 om deze prioriteit correct weer te geven.

#### P-5: Security by Design

**Assessment**: ✅ **Compliant**

**Evidence**:
- ✅ Dedicated PII Detection Pipeline
- ✅ AES-256 encryptie at rest
- ✅ TLS 1.3 voor in-transit
- ✅ PII logging voor AVG/GDPR compliance

**PII Pipeline**:
```rust
pub enum PIICategory {
    Email, PhoneNumber, SSN, IBAN, Name, Address, Custom(String)
}

pub struct WorkPayload {
    ScrubPII {
        content: String,
        pii_categories: Vec<PIICategory>,
    },
}
```

#### P-6: Observabiliteit

**Assessment**: ⚠️ **Partial Compliant**

**Aanwezig**:
- ✅ Tracing via trace_id (UUID)
- ✅ Logging in expert actors
- ✅ Heartbeat mechanism in registry

**Ontbrekend**:
- ❌ Distributed tracing strategy
- ❌ SLI/SLO definitions
- ❌ Expert performance metrics
- ❌ Centralized logging strategy

**Recommendation**: Voeg observability container toe aan architectuur.

---

## 4. Expert Mesh Architecture Review

### 4.1 Current Implementation Analysis

De codebase bevat een volledig geïmplementeerd expert mesh systeem dat **ontbreekt** in de huidige architectuurdocumentatie:

**Componenten**:
```
agent-service/src/mesh/
├── expert.rs           # Core Expert actor met capabilities
├── registry.rs         # Capability discovery registry
├── entry.rs            # Entry gateway voor requests
├── types.rs            # Shared types (ExpertMsg, WorkEnvelope)
└── experts/
    ├── frontend_expert.rs
    ├── research_expert.rs
    ├── reviewer_expert.rs
    ├── pii_stripper_expert.rs
    ├── rust_expert.rs
    └── schrijver_expert.rs
```

### 4.2 Expert Extensibility Framework

**Reeds Geïmplementeerd**:

1. **Capability-Based Discovery**
```rust
pub struct ExpertState {
    pub name: String,
    pub capabilities: Vec<String>,  // Strings als capability identifiers
    pub pending_tasks: HashMap<Uuid, PendingTask>,
}
```

2. **Registry Pattern**
```rust
pub struct RegistryActor {
    capabilities: HashMap<String, Vec<ActorRef<RegistryMsg>>>,
    health_map: HashMap<String, Instant>,
}
```

3. **Delegation Mechanism**
```rust
pub enum ExpertMsg {
    Work(WorkEnvelope),
    PeerResponse { trace_id: Uuid, result: String },
    SetPeers(Option<HashMap<String, ActorRef<ExpertMsg>>>),
}
```

**Extensibility Mechanismen**:

| Mechanisme | Status | Beschrijving |
|------------|--------|--------------|
| Capability Registration | ✅ Live | Experts registreren capabilities bij Registry |
| Peer Discovery | ✅ Live | PeerMap voor dynamisch peer lookup |
| Delegation Chain | ✅ Live | BatonPass met hop limits (MAX_DELEGATION_DEPTH) |
| Timeout & Recovery | ✅ Live | Timer tick met expired task recovery |
| Poison Pill | ✅ Live | Cascade cancellation via MeshSignal::Cancel |

### 4.3 Nieuwe Expert Toevoegen

**Stappen voor nieuwe expert**:

1. **Implementeer Expert trait/logic**:
```rust
pub struct NewExpert;

#[async_trait::async_trait]
impl Actor for NewExpert {
    type Msg = ExpertMsg;
    // Implementeer handle met WorkPayload processing
}
```

2. **Definieer capabilities**:
```rust
let capabilities = vec!["new_capability".to_string()];
```

3. **Registreer bij Registry**:
```rust
registry.cast(RegistryMsg::Register {
    actor: expert_ref,
    capability: "new_capability".to_string(),
    metadata: HashMap::new(),
})?;
```

### 4.4 Gaps in Documentatie

**Missing in HLD**:
- ❌ Expert Mesh container ontbreekt in DIAG-002
- ❌ Registry Actor niet zichtbaar
- ❌ Expert lifecycle (spawn/terminate) niet gedocumenteerd
- ❌ Capability discovery flow niet in diagrammen

**Required Addition**:

Nieuwe container in DIAG-002:
```
Container(expert_mesh, "Expert Mesh", "Rust, ractor", 
    "Uitbreidbaar expert agent systeem met capability discovery [Custom 0.40]")
```

---

## 5. Requirements Coverage Analysis

### 5.1 Functional Requirements Coverage

| Req ID | Requirement | Addressed | Design Element | Assessment |
|---------|-------------|-----------|----------------|------------|
| EFF-001 | Chat Gesprekken | ✅ | UI, API, DB | ✅ |
| EFF-002 | Berichten Verzenden | ✅ | UI, API, AI | ✅ |
| EFF-003 | Document Upload | ✅ | DocProc, Docs | ✅ |
| EFF-004 | PII Detectie | ✅ | PII Pipeline | ✅ |
| EFF-005 | Local-First Mode | ⚠️ | AI, Ollama | ⚠️ Needs clarification |
| EFF-006 | AI Provider Selectie | ✅ | AI Abstraction | ✅ |
| EFF-007 | Gesprek Zoeken | ✅ | DB, API | ✅ |
| EFF-008 | Gebruikersinstellingen | ✅ | DB, UI | ✅ |
| EFF-009 | Data Export | ✅ | DB, API | ✅ |
| EFF-010 | Data Verwijderen | ✅ | DB | ✅ |
| EFF-011/012 | Branding (Leiden/Utrecht) | ✅ | UI | ✅ |
| EFF-013 | Schaling (Utrecht) | ⚠️ | Alle containers | ⚠️ Needs validation |
| EFF-014 | Advanced Document Processing | ✅ | DocProc, Experts | ✅ |

**Coverage**: 13/14 (93%)

### 5.2 Non-Functional Requirements Coverage

| Req ID | Requirement | Target | HLD Approach | Assessment |
|---------|-------------|--------|--------------|------------|
| NFR-001 | Europese AI Modellen | EU-first | Ollama (local) > Mistral > Aleph | ✅ |
| NFR-002 | AVG/GDPR Compliance | Volledig | PII Pipeline, logging | ✅ |
| NFR-003 | Performance | <2s TTF | Local models, streaming | ✅ |
| NFR-004 | Beschikbaarheid | 99% | Graceful degradation | ✅ |
| NFR-005 | Security | Baseline | Encryption, PII | ✅ |
| TECH-001 | Local-First Architectuur | Local primary | ⚠️ Needs update | ⚠️ |
| TECH-003 | AI Provider Abstraktie | Uniform | AI Abstraction layer | ✅ |
| TECH-005 | Ollama Integration | Local | Ollama container | ✅ |
| TECH-007 | Document Pipeline | Parallel | DocProc with chunks | ✅ |
| TECH-008 | PII Pipeline | Dedicated | PII container | ✅ |

**Coverage**: 9/10 (90%)

### 5.3 Expert-Specific Requirements (Nieuw)

| Req ID | Requirement | Description | Status |
|---------|-------------|-------------|--------|
| TECH-009 | Expert Extensibility | Nieuwe experts toevoegen zonder code wijzigingen | ✅ Implemented |
| TECH-010 | Capability Discovery | Runtime expert discovery via registry | ✅ Implemented |
| TECH-011 | Expert Orchestration | Multi-expert workflows met delegation | ✅ Implemented |
| TECH-012 | Expert Resilience | Timeout, recovery, cascade cancellation | ✅ Implemented |

---

## 6. Local-First Strategy Clarification

### 6.1 Required Architecture Update

**Current Documentation (INCORRECT)**:
```
Rel(ai, mistral, "Primary AI provider", "HTTPS/TLS 1.3, API Key")
Rel(ai, aleph, "Secondary AI provider", "HTTPS/TLS 1.3, API Key")
Rel(ai, ollama, "Local fallback", "IPC")
```

**Corrected Architecture**:
```
Rel(ai, ollama, "Primary AI provider - Local models", "IPC")
Rel(ai, mistral, "Performance fallback - EU cloud", "HTTPS/TLS 1.3")
Rel(ai, aleph, "Secondary fallback - EU cloud", "HTTPS/TLS 1.3")
```

### 6.2 Fallback Decision Tree

```
┌─────────────────┐
│ User Request    │
└────────┬────────┘
         │
         v
┌─────────────────┐     YES    ┌──────────────┐
│ Local model     │────────────▶│ Use Ollama   │
│ available?      │             │ (Primary)    │
└────────┬────────┘             └──────────────┘
         │ NO
         v
┌─────────────────┐     YES    ┌──────────────┐
│ Performance     │────────────▶│ Use Mistral  │
│ threshold      │             │ (Fallback 1)  │
│ exceeded?      │             └──────────────┘
└────────┬────────┘
         │ NO
         v
┌─────────────────┐             ┌──────────────┐
│ Fallback to     │────────────▶│ Use Aleph    │
│ secondary EU    │             │ (Fallback 2)  │
└─────────────────┘             └──────────────┘
```

### 6.3 Performance Thresholds (TO DEFINE IN DLD)

| Scenario | Threshold | Action |
|----------|-----------|--------|
| Simple query | <3s local | Stay local |
| Complex query | >5s local | Fallback to cloud |
| Document >100p | N/A | Direct to cloud |
| Concurrent requests | >3 pending | Scale to cloud |

---

## 7. Issues and Recommendations

### 7.1 Critical Issues (BLOCKING)

| ID | Issue | Impact | Recommendation | Owner | Target |
|----|-------|--------|----------------|-------|--------|
| BLOCKING-01 | Expert Mesh ontbreekt in container diagram | Architectuur niet compleet | Voeg Expert Mesh container toe aan DIAG-002 | Architect | 2026-05-10 |
| BLOCKING-02 | Local-First strategy inconsistent | Verkeerde prioriteit in diagram | Update relaties: Ollama=primary, Mistral=fallback | Architect | 2026-05-10 |
| BLOCKING-03 | Registry niet gedocumenteerd | Expert discovery onduidelijk | Documenteer Registry pattern in HLD | Architect | 2026-05-10 |

### 7.2 High Priority Issues (ADVISORY)

| ID | Issue | Impact | Recommendation | Owner | Target |
|----|-------|--------|----------------|-------|--------|
| ADVISORY-01 | Observability voor expert mesh ontbreekt | Moeilijk troubleshooting | Voeg monitoring/logging strategy toe | SRE | Sprint 3 |
| ADVISORY-02 | Performance thresholds niet gedefinieerd | Geen duidelijke fallback criteria | Definieer TTF thresholds voor cloud fallback | Architect | Sprint 2 |
| ADVISORY-03 | Expert lifecycle management onduidelijk | Spawn/terminate niet gedocumenteerd | Documenteer expert lifecycle in DLD | Dev | Sprint 2 |
| ADVISORY-04 | Utrecht schalingsvereiste niet gevalideerd | Onzeker of 6k users ondersteund | Voer load testing uit voor Utrecht scenario | QA | Sprint 4 |

### 7.3 Low Priority Items (INFORMATIONAL)

| ID | Suggestion | Benefit |
|----|------------|---------|
| INFO-01 | Voeg sequence diagram toe voor expert workflows | Duidelijkere workflow visualisatie |
| INFO-02 | Documenteer expert capability naming conventions | Consistente nieuwe expert development |
| INFO-03 | Voeg Wardley Map toe voor expert positioning | Strategische component analyse |

---

## 8. Updated Architecture Recommendations

### 8.1 Recommended Container Structure (DIAG-002 Update)

```mermaid
C4Container
    title Container Diagram for Local-First AI Assistant (Updated)

    Person(ambtenaar, "Ambtenaar", "Gebruiker van de AI assistant")
    Person(dpo, "DPO", "Beheert privacy compliance")
    Person(ciso, "CISO", "Beheert security en infrastructuur")

    System_Ext(ollama, "Ollama", "Local model hosting - Primary AI provider")
    System_Ext(mistral, "Mistral AI", "Europese AI provider - Performance fallback")
    System_Ext(aleph, "Aleph Alpha", "Europese AI provider - Secondary fallback")

    System_Boundary(localassistant, "Local-First AI Assistant") {
        Container(desktop, "Desktop Application", "Tauri, Rust, TypeScript", "Cross-platform desktop shell")
        Container(ui, "Web UI", "React, TypeScript, TailwindCSS", "Gebruikersinterface [Custom 0.35]")
        Container(api, "API Service", "Rust, Actix-web", "RESTful API [Custom 0.42]")
        
        // NIEUW: Expert Mesh Container
        Container(expert_mesh, "Expert Mesh", "Rust, ractor", "Uitbreidbaar expert agent systeem [Custom 0.40]")
        
        Container(docproc, "Document Processing Pipeline", "Rust, pdf-extract", "Document parsing, chunking [Custom 0.40]")
        Container(pii, "PII Detection Pipeline", "Python, spaCy", "PII detectie en redactie [Custom 0.35]")
        Container(ai, "AI Provider Abstraction", "Rust, async traits", "Uniform AI interface [Custom 0.45]")
        ContainerDb(db, "Conversation Database", "SQLite", "Berichten, gesprekken [Commodity 0.95]")
        Container(docs, "Document Storage", "Filesystem", "Geüploade documenten [Commodity 0.90]")
    }

    // Updated relations with local-first priority
    Rel(ai, ollama, "Primary - Local models", "IPC")
    Rel(ai, mistral, "Fallback 1 - Performance trigger", "HTTPS/TLS 1.3")
    Rel(ai, aleph, "Fallback 2 - Secondary", "HTTPS/TLS 1.3")
    
    // Expert mesh integration
    Rel(api, expert_mesh, "Expert work delegation", "Function Call")
    Rel(expert_mesh, docproc, "Document processing", "Function Call")
    Rel(expert_mesh, pii, "PII scrubbing", "Function Call")
```

### 8.2 Expert Mesh Detail (New Component)

**Expert Mesh Container**:
- **Technology**: Rust, ractor actor framework
- **Purpose**: Uitbreidbaar multi-expert orchestration systeem
- **Evolution Stage**: Custom 0.40
- **Build/Buy**: BUILD

**Sub-components**:
- **EntryActor**: Gateway voor externe requests
- **RegistryActor**: Capability discovery en health monitoring
- **Expert Actors**: Domain experts (Research, Frontend, Rust, Schrijver, Reviewer, PII)
- **DocumentOrchestrator**: Multi-expert workflow voor document creatie
- **DocumentImprover**: Document verbetering workflow

**Expert Capabilities Mapping**:
| Expert | Capabilities | Description |
|--------|--------------|-------------|
| FrontendExpert | "frontend", "css", "html", "design" | Frontend technical expertise |
| ResearchExpert | "research", "http", "urls" | Web research en data extraction |
| ReviewerExpert | "review", "critique", "quality" | Content review en feedback |
| PIIStripperExpert | "pii", "privacy", "scrub" | PII detectie en redactie |
| RustExpert | "rust", "systems", "memory" | Rust systems programming |
| SchrijverExpert | "write", "content", "dutch" | Nederlandse content creatie |

---

## 9. Next Steps

### 9.1 Immediate Actions (Voor DLD)

1. **Update DIAG-002** met expert mesh container
2. **Corrigeer local-first priority** (Ollama = primary)
3. **Documenteer Registry pattern** in architectuur
4. **Definieer performance thresholds** voor cloud fallback

### 9.2 Detailed Design (DLD) Focus Areas

1. **Expert Mesh DLD**:
   - Expert lifecycle management
   - Capability registration protocol
   - Distributed tracing strategy
   - Error handling en recovery

2. **AI Provider Fallback DLD**:
   - Performance threshold definitions
   - Fallback decision tree
   - Fallback-back strategy (cloud → local)

3. **Observability DLD**:
   - Centralized logging
   - Expert performance metrics
   - Distributed tracing (trace_id propagation)

### 9.3 Follow-up Diagrams

1. **C4 Component Diagram** voor Expert Mesh
2. **Sequence Diagram** voor expert delegation flows
3. **Data Flow Diagram** voor PII redactie pipeline
4. **Deployment Diagram** voor multi-device scenario

---

## 10. Approval Decision

### 10.1 Final Decision

**Status**: ⚠️ **APPROVED WITH CONDITIONS**

**Conditions**:
1. BLOCKING-01, BLOCKING-02, BLOCKING-03 moeten opgelost zijn vóór DLD
2. ADVISORY items moeten aangepakt worden tijdens DLD

**Effective Date**: 2026-05-07 (onder voorwaarden)

### 10.2 Sign-Off

| Reviewer | Role | Decision | Date |
|----------|------|----------|------|
| [Enterprise Architect] | Lead Reviewer | ⚠️ Conditional | 2026-05-07 |
| [Security Architect] | Security Reviewer | ✅ Approve | [PENDING] |
| [Domain Expert] | Domain Reviewer | ⚠️ Conditional | [PENDING] |

---

## Appendix A: Expert Mesh Code References

**Bestanden**:
- `agent-service/src/mesh/expert.rs` - Core Expert actor
- `agent-service/src/mesh/registry.rs` - Capability registry
- `agent-service/src/mesh/entry.rs` - Entry gateway
- `agent-service/src/mesh/types.rs` - Shared types
- `agent-service/src/mesh/experts/*.rs` - Expert implementaties

**Key Types**:
```rust
pub struct ExpertState {
    pub name: String,
    pub capabilities: Vec<String>,
    pub pending_tasks: HashMap<Uuid, PendingTask>,
}

pub enum ExpertMsg {
    Work(WorkEnvelope),
    PeerResponse { trace_id: Uuid, result: String },
    SetPeers(Option<HashMap<String, ActorRef<ExpertMsg>>>),
}

pub struct WorkEnvelope {
    pub payload: WorkPayload,
    pub context: SessionContext,
    pub reply_to: Option<ActorRef<ExpertMsg>>,
    pub entry_reply: Option<ActorRef<EntryMsg>>,
    pub trace_id: Uuid,
    pub hop_count: u32,
}
```

---

**Gegenereerd door**: ArcKit `/arckit:hld-review` commando
**Gegenereerd op**: 2026-05-07
**ArcKit Versie**: 4.3.1
**Project**: Local-First AI Assistant (Project 000)
**AI Model**: Claude Opus 4.7
**Generation Context**: HLD review op basis van requirements (ARC-000-REQS-v1.0), principles (ARC-000-PRIN-v1.0), diagrammen (ARC-000-DIAG-001/002), en codebase analyse
