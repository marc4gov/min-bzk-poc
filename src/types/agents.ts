export type AgentMode = 'single' | 'multi';

/** Afgeleid van `agent-service` registry — niet handmatig uitbreiden hier; gebruik bij voorkeur `GET`-agents. */
export const SPECIALIST_IDS = [
  'code',
  'schrijf',
  'tools',
  'research',
  'translate',
  'analyze',
  'docs',
  'brainstorm',
  'data',
  'general',
] as const;

export type SpecialistId = (typeof SPECIALIST_IDS)[number];

export type SelectedAgent = 'auto' | 'orchestrator' | SpecialistId;

export interface AgentChatRequest {
  message: string;
  agent_mode: AgentMode;
  selected_agent?: SelectedAgent;
  /** Ollama-tag, bv. `qwen3-coder:latest`. Leeg = server gebruikt `OLLAMA_MODEL`/default. */
  model?: string;
}

export interface AgentChatResponse {
  content: string;
  agents_used: string[];
  tokens_used: number;
  /** Effectief gebruikt model op de agent-service. */
  model_used: string;
}

export interface AgentInfo {
  id: string;
  name: string;
  description: string;
  /** `chat` | `mesh` | `orchestrator` — agent-service `/api/agents`; oud zonder dit veld. */
  kind?: string;
  selectable?: boolean;
}

export interface AgentHealthResponse {
  status: string;
  ollama_available: boolean;
  openai_available: boolean;
}

export interface AgentStatus {
  id: string;
  name: string;
  state: 'idle' | 'thinking' | 'tool-using' | 'complete';
  progress?: number;
}

export type AgentEventType =
  | 'connected'
  | 'orchestrator_start'
  | 'orchestrator_decision'
  | 'agent_start'
  | 'agent_thinking'
  | 'agent_tool_use'
  | 'agent_tool_result'
  | 'agent_complete'
  | 'agent_error'
  | 'streaming'
  | 'mesh_step';

export interface AgentLogEntry {
  id: string;
  type: AgentEventType;
  timestamp: number;
  agent?: string;
  message: string;
  details?: Record<string, unknown>;
  tool?: string;
  tool_result?: unknown;
}

export interface AgentStreamingContent {
  agent: string;
  /** Cumulatieve tekst tot nu toe (tokenstream van de agent service). */
  content: string;
  done: boolean;
  /** Laatste chunk (optioneel, voor toekomstige UI). */
  delta?: string;
}

/**
 * Mesh-demo (`/api/mesh/demo`): Entry triage → experts.
 * Document-pipeline (`/api/mesh/demo/document`): alleen `DocumentOrchestrator` + keten.
 */
export interface MeshDemoResult {
  ok: boolean;
  result?: string;
  error?: string;
}
