# Local Assistant

Een privacy-vriendelijke AI assistent die volledig lokaal draait op je Mac. Geen data wordt verstuurd naar externe servers.

## Features

- 🚀 **Volledig lokaal** - Alle AI verwerking gebeurt op jouw machine
- 🔒 **Privacy first** - Je data verlaat jouw device nooit
- 💬 **Chatgeschiedenis** - Bewaar en beheer je gesprekken
- 📄 **Document verwerking** - Analyseer documenten lokaal
- ⌨️ **Global hotkey** - Cmd+Shift+A om snel de assistent te openen
- 🤖 **Multi-agent systeem** - Gespecialiseerde agents voor code, schrijven, en tools (optioneel)

## Quick Start

### 1. Installatie

```bash
# Dependencies installeren
npm install

# Dev server starten
npm run dev
```

### 2. Model Downloaden

**Optie A: Via de app**
Start de app en open het Model Manager scherm om een model te downloaden.

**Optie B: Via script**
```bash
./scripts/download-model.sh
```

### 3. Beschikbare Modellen

| Model | Grootte | Beschrijving |
|-------|---------|--------------|
| Mistral 7B (Q4) | 4.3 GB | Aanbevolen - balans tussen snelheid en kwaliteit |
| Phi-3 Mini (Q4) | 2.3 GB | Sneller - voor oudere Macs of beperkt RAM |
| Gemma 2B (Q4) | 1.6 GB | Kleinste - minimale resource usage |

## Development

```bash
# Dev mode
npm run dev

# Build voor productie
npm run build

# Type checking
npx tsc --noEmit

# Backend check (src-tauri directory)
cd src-tauri && cargo check
```

## Multi-Agent Systeem

Local Assistant ondersteunt een optioneel multi-agent systeem voor geavanceerde taken. Het systeem gebruikt gespecialiseerde agents die parallel kunnen werken:

### Beschikbare Agents

| Agent | Beschrijving |
|-------|-------------|
| **Orchestrator** | Analyseert je vraag en kiest de juiste agents |
| **Code** | Voor code generatie, debugging, en technische uitleg |
| **Schrijf** | Voor teksten schrijven, redigeren, en samenvatten |
| **Tools** | Voor bestanden lezen, web search, en shell commands |

### Multi-Agent Mode Starten

```bash
# Terminal 1: Start de agent service
cd agent-service
cargo run

# Terminal 2: Start de Tauri app
npm run dev
```

Als de agent service beschikbaar is, verschijnt er automatisch een selector in de chat interface om te kiezen tussen Single en Multi mode.

### Single vs Multi Mode

**Single Mode** (standaard)
- Eén LLM geeft direct antwoord
- Sneller voor eenvoudige vragen
- Werkt altijd, zelfs zonder agent service

**Multi Mode**
- Orchestrator verdeelt taken over gespecialiseerde agents
- Agents werken parallel voor snellere verwerking
- Ideaal voor complexe vragen die meerdere domeinen betreffen

### Cloud Fallback (Optioneel)

Je kunt optioneel OpenAI toevoegen als fallback voor betere kwaliteit op bepaalde taken:

```bash
export OPENAI_API_KEY=sk-...
cd agent-service && cargo run
```

De service blijft ook dan lokaal werken en gebruikt OpenAI alleen als dat voordelig is (bijvoorbeeld voor creatieve teksten).

## Architecture

```
localassistant/
├── src/                   # Frontend (React + TypeScript)
│   ├── components/        # UI components
│   ├── lib/              # API wrappers en utilities
│   └── types/            # TypeScript types
├── src-tauri/            # Backend (Rust)
│   ├── bridges/          # LLM inference engines
│   ├── commands/         # Tauri command handlers
│   └── services/         # Business logic
├── agent-service/        # Multi-agent microservice (Rust)
│   ├── src/              # Service source code
│   │   ├── agents/       # Agent implementations
│   │   ├── llm/          # LLM provider abstraction
│   │   ├── tools/        # Agent tools
│   │   └── api/          # REST API & WebSocket
│   └── tests/            # Integration tests
├── scripts/              # Helper scripts
└── models/               # Gedownloade modellen (wordt aangemaakt)
```

## Inference Engines

De app ondersteunt meerdere inference backends:

1. **llama-gguf** (default) - Native Rust crate, stil en snel
2. **Candle** - ML framework met subprocess fallback
3. **llama.cpp** - Directe integratie (in ontwikkeling)

## Troubleshooting

### Agent service niet beschikbaar
Als de Multi-Agent selector niet verschijnt:
1. Controleer of de agent service draait: `curl http://127.0.0.1:8080/health`
2. Start de service: `cd agent-service && cargo run`
3. De app toont automatisch de selector wanneer de service beschikbaar is

### Model wordt niet gevonden
Zorg dat het model in de juiste map staat:
- Development: `models/` in project root
- Productie: Naast de app bundle (macOS)

### Memory errors
Gebruik een kleiner model:
- Phi-3 Mini voor 8GB RAM
- Gemma 2B voor 4GB RAM

### Compileerfouten
```bash
# Cache legen en opnieuw bouwen
cd src-tauri
cargo clean
cargo build
```

## License

MIT
