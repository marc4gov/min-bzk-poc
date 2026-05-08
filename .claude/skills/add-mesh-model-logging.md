# add-mesh-model-logging

Voeg model-logging toe aan mesh experts in de agent-service, zodat in het SYSTEEM log zichtbaar is welk model gebruikt wordt.

## Wanneer gebruiken

Gebruik deze skill wanneer je:
- Een nieuwe mesh expert maakt die LLM calls doet via Ollama
- Een bestaande expert hebt die geen model-informatie toont in het log
- Wilt debuggen welk model effectief gebruikt wordt bij mesh operaties

## Achtergrond

De agent-service heeft een `ollama_bridge` module met twee functies:
- `chat_via_ollama_for_mesh()` - simpele LLM call
- `chat_via_ollama_for_mesh_with_events()` - LLM call **met** model-logging

Gebruik altijd de `with_events` versie voor experts die mesh_events hebben.

## Implementatie

### 1. Import de juiste functie

```rust
use crate::mesh::ollama_bridge::chat_via_ollama_for_mesh_with_events;
```

### 2. Roep de functie aan met mesh_events

```rust
match chat_via_ollama_for_mesh_with_events(
    &state.base.name,  // expert naam
    prompt,            // user prompt
    None,              // optionele model override
    &state.mesh_events, // <- dit is cruciaal voor logging
)
.await
{
    Ok(text) => { /* verwerk response */ }
    Err(e) => { /* handle fout */ }
}
```

### 3. Resultaat in SYSTEEM log

Na implementatie zie je in het log:

```
🕸️ Mesh  →  ExpertName: "Gebruikt model: digitsflow/bonsai-8b:latest"
```

## Voorbeeld expert

Hier is een complete expert met model-logging:

```rust
use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::sync::broadcast;

use crate::mesh::expert::{
    ExpertMsg, ExpertState, PeerMap, WorkEnvelope, WorkPayload,
};
use crate::mesh::types::MeshSignal;
use crate::mesh::ollama_bridge::chat_via_ollama_for_mesh_with_events;

pub struct MyExpert {
    base: ExpertState,
    mesh_events: Option<broadcast::Sender<crate::AgentEvent>>,
}

impl MyExpert {
    pub fn new() -> Self {
        let base = ExpertState::new(
            "MyExpert".to_string(),
            vec!["my-capability".to_string()],
        );
        Self { base, mesh_events: None }
    }

    async fn process_with_llm(&self, prompt: &str) -> Result<String, String> {
        chat_via_ollama_for_mesh_with_events(
            "MyExpert",
            prompt,
            None,
            &self.mesh_events,
        ).await
    }
}
```

## Belangrijke bestanden

- `agent-service/src/mesh/ollama_bridge.rs` - LLM bridge functions
- `agent-service/src/mesh/experts/` - Alle expert implementaties
- `agent-service/src/main.rs` - Poort config via `MESH_PORT`

## Environment variables

- `MESH_PORT` - Poort voor agent-service (default 8080, Tauri verwacht 8081)
- `MESH_USE_OLLAMA` - Zet op `0` om LLM uit te schakelen
- `OLLAMA_BASE_URL` - Ollama endpoint (default http://localhost:11434)
- `DEFAULT_OLLAMA_MODEL` - Standaard model

## Testing

Start de services:

```bash
# Terminal 1 - Agent service
cd agent-service
MESH_PORT=8081 cargo run --bin agent-service

# Terminal 2 - Tauri app
npm run tauri dev
```

Controleer in het SYSTEEM log of het model getoond wordt bij expert operaties.

## Gerelateerde documentatie

- `agent-service/src/mesh/mod.rs` - Mesh module exports
- `src-tauri/src/commands/agent.rs` - Tauri commands die mesh aanroepen
