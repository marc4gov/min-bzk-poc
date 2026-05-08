import type { AgentLogEntry, AgentStreamingContent } from '../types/agents';

const AGENT_WS_URL = 'ws://127.0.0.1:8080/ws';

type LogCallback = (entry: AgentLogEntry) => void;
type StreamCallback = (content: AgentStreamingContent) => void;
type ConnectedCallback = (connected: boolean) => void;

export class AgentWebSocket {
  private ws: WebSocket | null = null;
  private reconnectTimer: number | null = null;
  private messageBuffer: string[] = [];
  private isManualClose = false;

  constructor(
    private onLog: LogCallback,
    private onStream?: StreamCallback,
    private onConnected?: ConnectedCallback
  ) {}

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      this.ws = new WebSocket(AGENT_WS_URL);

      this.ws.onopen = () => {
        console.log('Agent WebSocket connected');
        this.isManualClose = false;
        this.onConnected?.(true);

        // Send buffered messages
        while (this.messageBuffer.length > 0 && this.ws?.readyState === WebSocket.OPEN) {
          const msg = this.messageBuffer.shift();
          if (msg) this.ws.send(msg);
        }
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.handleMessage(data);
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      this.ws.onclose = () => {
        console.log('Agent WebSocket disconnected');
        this.onConnected?.(false);
        this.ws = null;

        // Auto-reconnect if not manually closed
        if (!this.isManualClose) {
          this.scheduleReconnect();
        }
      };

      this.ws.onerror = (error) => {
        console.error('Agent WebSocket error:', error);
      };
    } catch (e) {
      console.error('Failed to create WebSocket:', e);
      this.scheduleReconnect();
    }
  }

  disconnect(): void {
    this.isManualClose = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer) return;

    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, 3000);
  }

  send(message: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(message);
    } else {
      this.messageBuffer.push(message);
    }
  }

  private handleMessage(data: unknown): void {
    if (!data || typeof data !== 'object') return;

    const event = data as Record<string, unknown>;

    switch (event.type) {
      case 'Connected':
        this.onLog({
          id: Date.now().toString(),
          type: 'connected',
          timestamp: Date.now(),
          message: 'Verbonden met agent service',
        });
        break;

      case 'Started':
        this.onLog({
          id: Date.now().toString(),
          type: event.agent === 'orchestrator' ? 'orchestrator_start' : 'agent_start',
          timestamp: (event.timestamp as number) || Date.now(),
          agent: event.agent as string,
          message: `Agent "${event.agent}" gestart`,
        });
        break;

      case 'OrchestratorDecision':
        this.onLog({
          id: Date.now().toString(),
          type: 'orchestrator_decision',
          timestamp: Date.now(),
          agent: 'orchestrator',
          message: `Routeert naar: ${event.target_agent}`,
          details: { reasoning: event.reasoning },
        });
        break;

      case 'Thinking':
        this.onLog({
          id: Date.now().toString(),
          type: 'agent_thinking',
          timestamp: (event.timestamp as number) || Date.now(),
          agent: event.agent as string,
          message: `Denkt na...`,
        });
        break;

      case 'ToolUse':
        this.onLog({
          id: Date.now().toString(),
          type: 'agent_tool_use',
          timestamp: (event.timestamp as number) || Date.now(),
          agent: event.agent as string,
          message: `Gebruikt tool: ${event.tool}`,
          tool: event.tool as string,
          details: { args: event.args },
        });
        break;

      case 'ToolResult':
        this.onLog({
          id: Date.now().toString(),
          type: 'agent_tool_result',
          timestamp: (event.timestamp as number) || Date.now(),
          agent: event.agent as string,
          message: `Tool resultaat: ${event.tool}`,
          tool: event.tool as string,
          tool_result: event.result,
        });
        break;

      case 'Response':
        this.onLog({
          id: Date.now().toString(),
          type: 'agent_complete',
          timestamp: (event.timestamp as number) || Date.now(),
          agent: event.agent as string,
          message: 'Antwoord gereed',
          details: { content_preview: (event.content as string)?.substring(0, 100) },
        });

        // Also trigger streaming callback
        this.onStream?.({
          agent: event.agent as string,
          content: event.content as string,
          done: true,
        });
        break;

      case 'Error':
        this.onLog({
          id: Date.now().toString(),
          type: 'agent_error',
          timestamp: (event.timestamp as number) || Date.now(),
          agent: event.agent as string,
          message: `Fout: ${event.error}`,
        });
        break;

      case 'Mesh': {
        const tsRaw = event.timestamp as number | undefined;
        const tsMs =
          typeof tsRaw === 'number'
            ? tsRaw < 2e12
              ? tsRaw * 1000
              : tsRaw
            : Date.now();
        this.onLog({
          id: `${Date.now()}-${Math.random().toString(36).slice(2, 10)}`,
          type: 'mesh_step',
          timestamp: tsMs,
          agent: (event.actor as string) || 'mesh',
          message: `${String(event.phase ?? '')}${event.detail != null && String(event.detail) !== '' ? `: ${event.detail}` : ''}`,
          details: { model: event.model as string | undefined },
        });
        break;
      }

      case 'Streaming':
        this.onLog({
          id: Date.now().toString(),
          type: 'streaming',
          timestamp: Date.now(),
          agent: event.agent as string,
          message: '',
          details: { delta: event.delta },
        });

        this.onStream?.({
          agent: event.agent as string,
          content: event.content as string,
          done: event.done as boolean,
          delta: typeof event.delta === 'string' ? event.delta : undefined,
        });
        break;

      default:
        console.log('Unknown event type:', event);
    }
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}
