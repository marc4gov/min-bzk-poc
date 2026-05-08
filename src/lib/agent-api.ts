import { invoke } from '@tauri-apps/api/core';
import type {
  AgentChatRequest,
  AgentChatResponse,
  AgentInfo,
  AgentHealthResponse,
  MeshDemoResult,
} from '../types/agents';

const withTimeout = <T>(promise: Promise<T>, ms: number, errorMessage: string): Promise<T> => {
  return Promise.race([
    promise,
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new Error(errorMessage)), ms)
    ),
  ]);
};

export const agentApi = {
  sendMessage: async (request: AgentChatRequest): Promise<AgentChatResponse> => {
    const backendRequest = {
      message: request.message,
      selected_agent: request.agent_mode === 'multi' && request.selected_agent !== 'auto'
        ? request.selected_agent
        : undefined,
      ...(request.model?.trim()
        ? { model: request.model!.trim() }
        : {}),
    };

    return withTimeout(
      invoke('agent_chat', { request: backendRequest }),
      120000,
      'Agent service reageert niet (time-out). Controleer of de agent service draait.'
    );
  },

  getAgents: async (): Promise<AgentInfo[]> => {
    return await invoke('list_agents');
  },

  /** Lokale Ollama-modellen via agent-service (`/api/tags`). */
  listOllamaModels: async (): Promise<{ ok: boolean; models: { name: string }[] }> => {
    return await invoke('ollama_models');
  },

  getHealth: async (): Promise<AgentHealthResponse> => {
    return await invoke('agent_health');
  },

  isServiceAvailable: async (): Promise<boolean> => {
    try {
      const health = await agentApi.getHealth();
      return health.status === 'healthy';
    } catch {
      return false;
    }
  },

  /**
   * Mesh-demo: `/api/mesh/demo` registreert alleen RustExpert + FrontendExpert (delegatie tussen peers).
   * Chat-specialisten uit `GET /api/agents` worden hier niet gebruikt.
   */
  runMeshDemo: async (
    query: string,
    timeoutSecs = 90,
    fullMesh = false
  ): Promise<MeshDemoResult> => {
    return invoke<MeshDemoResult>('mesh_demo', {
      request: {
        query,
        timeout_secs: Math.min(600, Math.max(5, timeoutSecs)),
        full_mesh: fullMesh,
      },
    });
  },

  /** DocumentOrchestrator: Research → Schrijven → PII scrub → Review (`POST /api/mesh/demo/document`). */
  runMeshDocument: async (urls: string[], timeoutSecs = 120): Promise<MeshDemoResult> => {
    const list = urls.map((u) => u.trim()).filter(Boolean);
    return invoke<MeshDemoResult>('mesh_document', {
      request: {
        urls: list,
        timeout_secs: Math.min(600, Math.max(15, timeoutSecs)),
      },
    });
  },

  /** Document verbeteren met mesh pipeline: content → verbetering (`POST /api/mesh/improve`). */
  improveDocument: async (
    content: string,
    filename: string,
    instructions: string,
    model?: string,
    timeoutSecs = 120
  ): Promise<MeshDemoResult> => {
    return invoke<MeshDemoResult>('mesh_improve_document', {
      request: {
        content,
        filename,
        instructions,
        model,
        timeout_secs: Math.min(600, Math.max(15, timeoutSecs)),
      },
    });
  },
};
