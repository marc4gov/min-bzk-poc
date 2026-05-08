# PII Detection via LLM - Implementatie

## Overzicht

De PIIStripperExpert is aangepast om een LLM (via Ollama) te gebruiken voor PII detection in plaats van alleen regex of een aparte privacy-filter service.

## Wat is veranderd

### 1. PIIStripperExpert (`agent-service/src/mesh/experts/pii_stripper_expert.rs`)

**Nieuwe features:**
- LLM-based PII detection via `chat_via_ollama_for_mesh_with_events()`
- Structured JSON output van de LLM met gevonden PII entities
- Fallback mechanism: LLM → privacy-filter → regex

**Nieuwe environment variable:**
- `MESH_PII_MODE` - Kies de PII detection modus:
  - `llm` - Gebruik Ollama LLM (standaard)
  - `privacy_filter` - Gebruik privacy-filter sidecar
  - `regex` - Alleen regex fallback

**Log output:**
```
🕸️ Mesh  →  PIIStripperExpert: "Gebruikt model: digitsflow/bonsai-8b:latest"
🕸️ Mesh  →  scrub_klaar: "LLM PII-detectie via Ollama..."
```

### 2. Model logging voor alle experts

Experts die LLM gebruiken tonen nu welk model effectief gebruikt wordt:

**Aangepaste experts:**
- `PIIStripperExpert` - LLM PII detection ✅
- `FrontendExpert` - Frontend advies ✅
- `RustExpert` - Rust code advies ✅

**Nog niet aangepast:**
- `SchrijverExpert` - Geen LLM in huidige versie
- `ResearchExpert` - Gebruikt HTTP, geen LLM
- `ReviewerExpert` - Geen LLM

## Service configuratie

### Poorten

| Service | Poort | Opmerking |
|---------|-------|-----------|
| Agent Service | 8081 | Tauri verwacht deze poort |
| Ollama | 11434 | Standaard Ollama poort |
| Frontend dev | 1420 | Vite dev server |

### Environment variables

```bash
# Agent service poort (Tauri verwacht 8081!)
export MESH_PORT=8081

# PII detection modus
export MESH_PII_MODE=llm  # llm, privacy_filter, of regex

# Ollama config
export OLLAMA_BASE_URL=http://localhost:11434
export DEFAULT_OLLAMA_MODEL=digitsflow/bonsai-8b:latest

# Oude privacy-filter (niet meer nodig bij LLM modus)
export PII_PRIVACY_FILTER_URL=http://127.0.0.1:8091
```

## Services starten

```bash
# Terminal 1 - Agent service
cd /Users/marc/Projecten/localassistant/agent-service
MESH_PORT=8081 cargo run --bin agent-service

# Terminal 2 - Tauri app
cd /Users/marc/Projecten/localassistant
npm run tauri dev
```

## Controleren of het werkt

1. Open de app
2. Upload een document met PII (email, namen, etc.)
3. Check SYSTEEM log voor:
   - `PIIStripperExpert: "Gebruikt model: <model>"`
   - Model-naam verschijnt bij elke expert die LLM gebruikt

## Bestanden

| Bestand | Wijziging |
|---------|-----------|
| `agent-service/src/mesh/experts/pii_stripper_expert.rs` | LLM PII detection + model logging |
| `agent-service/src/mesh/experts/frontend_expert.rs` | Model logging toegevoegd |
| `agent-service/src/mesh/experts/rust_expert.rs` | Model logging toegevoegd |
| `.claude/skills/add-mesh-model-logging.md` | Skill voor toekomstige experts |

## Toekomstige verbeteringen

- [ ] SchrijverExpert LLM integratie met model logging
- [ ] ReviewerExpert LLM integratie
- [ ] Metrics toevoegen voor model gebruik per expert
- [ ] Configurable prompts per expert
