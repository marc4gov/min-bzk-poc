# Lokale AI Assistent - Design Spec

**Date:** 2026-04-22
**Status:** Approved
**Version:** 1.0

## Overview

Een desktop AI-assistent gebouwd met **Tauri 2.0**, **MLX** voor Apple Silicon inference, en een quantized **Mistral 7B** model. De app levert productiviteit tools: chatgeschiedenis, document analyse, global hotkey access, en templates. Het model is gebundeld met de app voor een simpele installatie.

### Tech Stack

| Component | Technology |
|-----------|------------|
| Frontend | TypeScript + React + Tailwind CSS |
| Backend | Rust Tauri 2.0 |
| Inference | MLX (via Swift bindings) with abstraction for future llama.cpp |
| Model | Mistral 7B Q4_K_M (~4GB) |
| Database | SQLite |

### Key Trade-off

Grotere app download (~500MB-1GB) voor zero-config experience.

## Architecture

### High-Level Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React + TS)                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │  Chat UI ││Documents ││Templates ││  Global Hotkey   │ │
│  │  View    ││  Panel   ││ Manager  ││   (Tauri API)    │ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────────────────┘ │
└───────┼────────────┼────────────┼──────────────────────────────┘
        │            │            │
        └────────────┴────────────┴───────────┐
                                              │
                    ┌─────────────────────────▼───────────────────┐
                    │            Tauri IPC Layer                   │
                    │  Commands: chat, document, config, history  │
                    └─────────────────────────┬───────────────────┘
                                              │
        ┌─────────────────────────────────────┼───────────────────────────┐
        │                                     │                           │
┌───────▼───────────┐              ┌─────────▼─────────┐   ┌─────────────▼──────────┐
│  Inference Engine │              │  Storage Service  │   │   System Integration   │
│  (MLX Module)     │              │  - SQLite         │   │   - Global Hotkey      │
│  - Model loading  │              │  - Chat history   │   │   - File dialogs       │
│  - Tokenization   │              │  - Documents      │   │   - Notifications      │
│  - Generation     │              │  - Templates      │   │                        │
└───────────────────┘              └───────────────────┘   └────────────────────────┘
```

### Communication Pattern

Frontend-Backend communicatie via Tauri Events (`invoke`, `emit`, `listen`).

### Inference Abstraction

Een Rust trait `InferenceEngine` waarvan `MLXEngine` de eerste implementatie is. Later kan `LlamaCppEngine` toegevoegd worden voor cross-platform support.

```rust
trait InferenceEngine: Send + Sync {
    async fn load_model(&mut self, path: &Path) -> Result<()>;
    async fn generate(&self, prompt: &str, params: GenerationParams)
        -> Pin<Box<dyn Stream<Item = String> + Send>>;
    async fn unload(&mut self) -> Result<()>;
}
```

## Components

### Frontend Components

| Component | Verantwoordelijkheid |
|-----------|---------------------|
| **ChatView** | Hoofd chat interface met berichten stream, markdown rendering, stop generation |
| **DocumentPanel** | Bestand analyse met drag-drop, preview, extraction |
| **TemplateManager** | Sjablonen beheer met voorgedefinieerde en user prompts |
| **HistorySidebar** | Gesprekken lijst met search en export/restore |

### Backend Services (Rust)

| Service | Verantwoordelijkheid |
|---------|---------------------|
| **InferenceService** | AI generation, model lazy-loading, streaming responses |
| **StorageService** | SQLite database, document indexing |
| **HotkeyService** | Global shortcuts (Cmd+Shift+A, Cmd+Shift+V) |
| **ConfigService** | Model parameters, theme preferences |

## Data Flows

### Chat Generation

```
User types message
       │
       ▼
Frontend validates ──► Error toast
       │
       ▼
invoke('chat_generate', { message, history_id })
       │
       ▼
InferenceService.receive()
       │
       ├─► Load history from StorageService
       ├─► Build prompt (system + history + user)
       │
       ▼
MLXEngine.generate(prompt) ──► Model not loaded? ──► Load first
       │
       ▼
Stream tokens via Tauri Events
       │
       ▼
Frontend appends to message (streaming UI)
       │
       ▼
Generation complete ──► Save to SQLite
```

### Document Analysis

```
User drops file
       │
       ▼
invoke('document_extract', { file_path })
       │
       ▼
StorageService.extract_text()
       ├─► PDF: pdf extractor
       ├─► DOCX: docx parser
       └─► txt/md: direct read
       │
       ▼
Return text preview to frontend
       │
       ▼
User confirms "Analyseer dit"
       │
       ▼
invoke('document_analyze', { text, prompt })
       │
       ▼
Same generation flow as chat
```

### Global Hotkey

```
User presses Cmd+Shift+A
       │
       ▼
Tauri global shortcut listener
       │
       ▼
emit('hotkey_toggle')
       │
       ▼
Frontend: window.show() / window.hide()
```

## Error Handling

### Inference Errors

| Error | Handling |
|-------|----------|
| Model not found | Fallback download prompt of instructies |
| Out of memory | Stop generation, toon error met suggesties |
| Generation timeout (30s) | Cancel en toon timeout message |
| Invalid model format | Error met model version info |

### I/O Errors

| Error | Handling |
|-------|----------|
| File read fails | User-friendly "Kon bestand niet lezen" |
| SQLite locked | Retry met backoff (max 3x) |
| Disk full | Critical error, toon storage info |

### Recovery Strategy

- Model corruption: Auto-re-download van bundled model
- Database corruption: Create new DB, preserve old voor recovery
- Crash on generation: Log stack trace, offer "Safe mode"

## UI Design

### Style: Modern Tech

- Dark theme met accent kleuren
- Hoge contrast voor leesbaarheid
- Subtiele animaties voor state changes
- Native macOS window controls

### Key Screens

1. **Main Chat Window** - Chat input, message history, sidebar toggle
2. **Quick Assist Modal** - Triggered via hotkey, minimal UI
3. **Settings Panel** - Model config, templates, key bindings

## Testing Strategy

### Unit Tests (Rust)

- `InferenceService` - Mock MLX engine, test prompt building
- `StorageService` - In-memory SQLite, test CRUD
- `HotkeyService` - Test registration, callbacks

### Integration Tests

- Tauri command handlers - Full IPC roundtrip
- Model loading - Met klein mock model
- Document extraction - Sample files

### Frontend Tests

- Component rendering - React Testing Library
- User interactions - Chat, upload, hotkey UI
- State management - Chat history updates

### Performance Targets

| Metric | Target |
|--------|--------|
| Cold start (zonder model) | <2s |
| First token latency | <500ms |
| Memory usage (Mistral 7B Q4) | <6GB |

## Implementation Phases

1. **Phase 1:** Core Tauri setup + MLX integration proof-of-concept
2. **Phase 2:** Basic chat UI + inference service
3. **Phase 3:** Storage, history, templates
4. **Phase 4:** Document analysis, hotkey integration
5. **Phase 5:** Polish, testing, packaging
