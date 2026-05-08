# Agent Service

Multi-agent microservice for Local Assistant, built with Rust.

## Features

- **Orchestrator Agent**: Analyzes queries and routes to appropriate specialized agents
- **Code Agent**: Specialized in code generation and debugging
- **Schrijf Agent**: Dutch writing assistant for text creation and editing
- **Tools Agent**: File system, web search, and shell command access
- **Hybrid LLM Backend**: Ollama (local) + OpenAI (cloud fallback)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    HTTP API (Axum)                               │
│  POST /api/chat  GET /api/agents  GET /health  /ws (WebSocket) │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Orchestrator Agent                             │
│    Analyzes query → Distributes tasks to specialized agents     │
└─────────────────────────────────────────────────────────────────┘
                            │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
   ┌─────────┐      ┌──────────┐      ┌──────────┐
   │  Code   │      │ Schrijf  │      │  Tools   │
   │  Agent  │      │  Agent   │      │  Agent   │
   └─────────┘      └──────────┘      └──────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
              ┌────────────────────────┐
              │    LLM Provider Layer   │
              │  Ollama      OpenAI     │
              └────────────────────────┘
```

## Running

### Development

```bash
cd agent-service
cargo run
```

The service will start on http://127.0.0.1:8080

### Production

```bash
cd agent-service
cargo build --release
./target/release/agent-service
```

## API Endpoints

### Health Check

Check if the service is running and which LLM backends are available.

```bash
curl http://127.0.0.1:8080/health
```

Response:
```json
{
  "status": "healthy",
  "ollama_available": true,
  "openai_available": false
}
```

### List Agents

Get information about available agents.

```bash
curl http://127.0.0.1:8080/api/agents
```

Response:
```json
[
  {
    "id": "orchestrator",
    "name": "Orchestrator",
    "description": "Analyseert je vraag en kiest de juiste agents"
  },
  {
    "id": "code",
    "name": "Code Agent",
    "description": "Schrijft en analyseert code"
  },
  {
    "id": "schrijf",
    "name": "Schrijf Agent",
    "description": "Helpt met teksten schrijven en redigeren"
  },
  {
    "id": "tools",
    "name": "Tools Agent",
    "description": "Gebruikt tools zoals bestanden lezen en web search"
  }
]
```

### Chat (Single or Multi-Agent)

Send a message to the agent system.

```bash
curl -X POST http://127.0.0.1:8080/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Schrijf een Python functie om een CSV te lezen",
    "agent_mode": "multi",
    "selected_agent": "auto"
  }'
```

Request parameters:
- `message`: The user's message
- `agent_mode`: `"single"` or `"multi"`
- `selected_agent`: Optional - `"auto"`, `"code"`, `"schrijf"`, `"tools"`, `"single"`

Response:
```json
{
  "content": "Hier is een Python functie...",
  "agents_used": ["code", "schrijf"],
  "tokens_used": 342
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OLLAMA_BASE_URL` | Ollama API URL | `http://localhost:11434` |
| `OPENAI_API_KEY` | OpenAI API key (optional) | - |

## Agent Routing Logic

The Orchestrator uses keyword analysis to route requests:

| Keywords | Agent |
|----------|-------|
| code, function, bug, debug, python, rust, javascript | Code |
| schrijf, tekst, mail, brief, essay, verhaal, samenvatten | Schrijf |
| bestand, lees, zoek, search, file, web, directory | Tools |

If no keywords match, defaults to Schrijf agent.

## LLM Selection

| Condition | Backend |
|-----------|---------|
| Short prompts (<500 chars) | Ollama (speed) |
| Code keywords | Ollama |
| Creative writing | OpenAI (if available) |
| Default | Ollama |

## Development

### Run tests

```bash
cargo test
```

### Run integration tests (requires Ollama running)

```bash
cargo test --ignored
```

### Check code

```bash
cargo check
cargo clippy
```

## Troubleshooting

### Service not responding

1. Check if the service is running: `curl http://127.0.0.1:8080/health`
2. Check logs for errors
3. Ensure port 8080 is not in use

### Ollama connection failed

1. Ensure Ollama is running: `ollama list`
2. Check OLLAMA_BASE_URL if using custom port
3. Verify Ollama API is accessible: `curl http://localhost:11434/api/tags`

### OpenAI errors

1. Set OPENAI_API_KEY environment variable
2. Verify API key is valid
3. Service will fall back to Ollama if OpenAI fails

## License

MIT
