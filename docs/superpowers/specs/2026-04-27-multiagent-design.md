# Multi-Agent Systeem Design - Local Assistant

**Date:** 2026-04-27
**Status:** Approved
**Version:** 1.0

## Overview

Uitbreiding van Local Assistant met een multi-agent systeem gebaseerd op AutoAgents (Rust framework). Het systeem ondersteunt gespecialiseerde agents die parallel kunnen werken, tools kunnen gebruiken, en automatisch of handmatig kunnen worden aangestuurd.

### Tech Stack Integration

| Component | Technology |
|-----------|------------|
| Multi-Agent Framework | AutoAgents (Rust) |
| Deployment | Separate microservice |
| LLM Backend | Hybrid: Ollama (lokaal) + OpenAI (cloud) |
| Communication | Tauri IPC → HTTP/gRPC → AutoAgents |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Frontend (React)                            │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Agent Selector  │  Chat Area  │  Agent Status Panel       │    │
│  └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Tauri IPC Layer                               │
│              Commands: agent_chat, agent_status, list_agents        │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  AutoAgents Microservice (Rust)                     │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │              Environment + Runtime                          │    │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐           │    │
│  │  │ Orchestr.  │  │   Code     │  │  Schrijf   │           │    │
│  │  │  Agent     │  │  Agent     │  │  Agent     │  ...       │    │
│  │  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘           │    │
│  └────────┼──────────────┼──────────────┼─────────────────────┘    │
│           │              │              │                           │
│           └──────────────┴──────────────┴───────────┐               │
│                      │ Topics (Events)              │               │
│                      ▼                              │               │
│  ┌─────────────────────────────────────────────────────┐           │
│  │              LLM Provider Layer                     │           │
│  │  ┌──────────────┐  ┌──────────────────────────┐    │           │
│  │  │  Ollama      │  │  OpenAI-Compatible API   │    │           │
│  │  └──────────────┘  └──────────────────────────┘    │           │
│  └─────────────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────────────┘
```

## Components

### Orchestrator Agent
- **Rol**: Centrale dispatcher die analyseert welke agents nodig zijn
- **Input**: User query
- **Output**: Gedistribueerde taken naar specifieke topics
- **Tools**: Agent classifier, task planner
- **Executor**: ReActAgent met reasoning

### Code Agent
- **Rol**: Code generatie, debugging, technische uitleg
- **Topic**: `code.tasks`
- **Tools**: Syntax checker, code formatter
- **Memory**: SlidingWindowMemory (window=50)
- **LLM**: Voorkeur lokaal (snellere iteratie)

### Schrijf Agent
- **Rol**: Tekst schrijven, redigeren, samenvatten
- **Topic**: `schrijf.tasks`
- **Tools**: Spell checker, style guide
- **Memory**: SlidingWindowMemory (window=20)
- **LLM**: Voorkeur OpenAI (betere kwaliteit)

### Tools Agent
- **Rol**: File system, web search, API calls
- **Topic**: `tools.tasks`
- **Tools**:
  - `file_read`: Lees lokale bestanden
  - `web_search`: DuckDuckGo search
  - `shell_execute`: Veilige shell commands
- **LLM**: Lokaal (security considerations)

### LLM Provider Layer

**OllamaBridge**
- Verbinding met lokale Ollama instance
- Modellen: llama3, mistral, codellama
- Fallback: OpenAI als lokaal unavailable

**OpenAIBridge**
- OpenAI API voor complexe taken
- GPT-4o voor kwaliteit, GPT-4o-mini voor snelheid

**Routing Heuristiek**
- < 100 tokens: Lokaal (snelheid)
- Code vragen: Lokaal (codellama)
- Creatieve tekst: Cloud (kwaliteit)
- Parallel tasks: Mix (snelheid + kwaliteit)

## Data Flows

### User Chat Flow

```
User: "Schrijf een Python functie om een CSV te lezen en geef uitleg"
       │
       ▼
Tauri IPC: agent_chat({ message, selected_agent: "auto" })
       │
       ▼
Microservice HTTP: POST /api/chat
       │
       ▼
Runtime.publish("orchestrator.in", Task::new(message))
       │
       ▼
Orchestrator Agent analyseert → "Code + Schrijf nodig"
       │
       ├─► publish("code.tasks", "Genereer Python CSV functie")
       └─► publish("schrijf.tasks", "Geef uitleg bij de code")
```

### Parallel Agent Processing

```
┌─────────────────┐         ┌─────────────────┐
│   Code Agent    │         │  Schrijf Agent  │
└────────┬────────┘         └────────┬────────┘
         │                          │
         ▼                          ▼
  LLM generate (Ollama)      LLM generate (OpenAI)
         │                          │
         ▼                          ▼
  emit AgentResponse      emit AgentResponse
         │                          │
         └──────────────┬───────────┘
                        │
                        ▼
              Orchestrator merge
                        │
                        ▼
              emit FinalResponse
                        │
                        ▼
              Frontend streams updates
```

### Event Stream to Frontend

```
AutoAgents Events ──► Microservice WebSocket ──► Tauri Event ──► React
```

Events:
- `AgentStarted { agent_id }`
- `AgentProgress { agent_id, delta }`
- `ToolCall { tool_name, args }`
- `AgentComplete { agent_id, result }`
- `OrchestrationComplete { final_response }`

## Error Handling

| Situatie | Afhandeling |
|----------|-------------|
| LLM niet beschikbaar (lokaal) | Fallback naar OpenAI API |
| OpenAI API error | Fallback naar lokaal model |
| Beide unavailable | Retourneer foutmelding aan UI |
| Agent timeout (30s) | Retourneer gedeeltelijk resultaat + foutmelding |
| Tool execute faalt | Log error + ga door met andere tools |
| Microservice onbereikbaar | Frontend toont "Agenten offline" - fallback naar bestaande single-agent mode |

### Retry Strategy
- **LLM calls**: 3 retries met exponential backoff (1s, 2s, 4s)
- **Tool calls**: 1 retry, daarna skip met warning
- **Agent orchestration**: Geen retry - orchestrator bepaalt alternatief

## Frontend Changes

### New Components

| Component | Doel |
|-----------|------|
| `AgentSelector` | Dropdown voor agent selectie (Auto, Code, Schrijf, Tools) |
| `AgentStatusPanel` | Toont actieve agents en hun status |
| `AgentMessage` | Bericht component met agent-indicator |

### UI State

```typescript
type AgentMode = 'single' | 'multi';
type SelectedAgent = 'auto' | 'orchestrator' | 'code' | 'schrijf' | 'tools';

interface AgentStatus {
  id: string;
  name: string;
  state: 'idle' | 'thinking' | 'tool-using' | 'complete';
  progress?: number;
}
```

## Microservice API

### REST Endpoints

```
POST /api/chat
  Body: { message, agent_mode, selected_agent }
  Response: stream of Server-Sent Events

GET /api/agents
  Response: [{ id, name, description, status }]

GET /api/health
  Response: { status, ollama_available, openai_available }
```

### WebSocket Events

```typescript
// Client → Server
{ type: "chat", message: string, agent: string }

// Server → Client
{ type: "agent_start", agent: string }
{ type: "agent_delta", agent: string, content: string }
{ type: "tool_call", tool: string, args: object }
{ type: "agent_complete", agent: string }
{ type: "complete", response: string, agents_used: string[] }
```

## Testing Strategy

### Unit Tests
- Agent tools los testen met mocked LLM
- LLM provider switching logic
- Event serialization/deserialization

### Integration Tests
- Volledige flow met echte Ollama instance
- OpenAI API integration (met test key)
- Topic routing en event streaming

### E2E Tests
- Frontend → Tauri → Microservice → Agents
- Agent timeout en fallback scenarios
- Parallel processing verification

### Performance Tests
- Token throughput per agent type
- Parallel vs sequential processing speedup
- Memory usage met meerdere agents

## Implementation Phases

1. **Phase 1: Microservice Foundation**
   - AutoAgents project setup
   - LLM provider abstraction
   - Basic REST API

2. **Phase 2: Core Agents**
   - Orchestrator agent
   - Code agent (met basis tools)
   - Schrijf agent

3. **Phase 3: Tools & Integration**
   - Tools agent met file/web/shell
   - Tauri IPC integration
   - Frontend agent selector

4. **Phase 4: Polish & Testing**
   - Event streaming UI
   - Error handling
   - Performance optimization

## Migration Notes

- Bestaande single-agent mode blijft beschikbaar als fallback
- Oudere chats blijven werken
- Microservice is optionele feature (kan uitgeschakeld worden)
