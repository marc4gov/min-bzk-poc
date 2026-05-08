import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type MutableRefObject,
  type ReactNode,
} from 'react';
import { AgentLogPanel } from './components/AgentLogPanel';
import { AgentWebSocket } from './lib/agent-websocket';
import { agentApi } from './lib/agent-api';
import type { AgentLogEntry, AgentStreamingContent } from './types/agents';

export type AgentDiagnosticsContextValue = {
  agentServiceAvailable: boolean;
  /** ChatView zet dit op stream-updates; null wanneer Chat geen actieve stream heeft. */
  streamRelayRef: MutableRefObject<((stream: AgentStreamingContent) => void) | null>;
  clearAgentLogs: () => void;
  logPanelVisible: boolean;
  setLogPanelVisible: (v: boolean | ((prev: boolean) => boolean)) => void;
  /** Handmatig opnieuw health + optioneel WebSocket (Mesh-knop). */
  recheckAgentService: () => Promise<void>;
};

const AgentDiagnosticsContext = createContext<AgentDiagnosticsContextValue | null>(null);

export function useAgentDiagnostics(): AgentDiagnosticsContextValue {
  const ctx = useContext(AgentDiagnosticsContext);
  if (!ctx) {
    throw new Error('useAgentDiagnostics must be used within AgentDiagnosticsProvider');
  }
  return ctx;
}

/**
 * Eén WebSocket naar agent-service en één vast logpaneel — buiten `main [overflow]`
 * zodat `position: fixed` op alle tabs (incl. Mesh) correct werkt.
 */
export function AgentDiagnosticsProvider({ children }: { children: ReactNode }) {
  const [agentLogs, setAgentLogs] = useState<AgentLogEntry[]>([]);
  const [logPanelVisible, setLogPanelVisible] = useState(true);
  const [agentServiceAvailable, setAgentServiceAvailable] = useState(false);

  const streamRelayRef = useRef<((stream: AgentStreamingContent) => void) | null>(null);
  const wsRef = useRef<AgentWebSocket | null>(null);

  const clearAgentLogs = useCallback(() => setAgentLogs([]), []);

  const connectWebSocketOnce = useCallback(() => {
    if (wsRef.current) return;
    wsRef.current = new AgentWebSocket(
      (entry) => {
        setAgentLogs((prev) => [...prev, entry]);
      },
      (stream) => {
        streamRelayRef.current?.(stream);
      },
      (connected) => {
        if (!connected && import.meta.env.DEV) {
          console.log('Agent WebSocket disconnected');
        }
      }
    );
    wsRef.current.connect();
  }, []);

  const recheckAgentService = useCallback(async () => {
    try {
      const available = await agentApi.isServiceAvailable();
      setAgentServiceAvailable(available);
      if (available) {
        connectWebSocketOnce();
      }
    } catch {
      setAgentServiceAvailable(false);
    }
  }, [connectWebSocketOnce]);

  useEffect(() => {
    let cancelled = false;
    let retryTimer: number | undefined;

    const checkAvailability = async (): Promise<boolean> => {
      try {
        const available = await agentApi.isServiceAvailable();
        if (cancelled) return false;
        setAgentServiceAvailable(available);
        if (available) {
          connectWebSocketOnce();
        }
        return available;
      } catch {
        if (!cancelled) setAgentServiceAvailable(false);
        return false;
      }
    };

    const scheduleRetry = () => {
      retryTimer = window.setTimeout(async () => {
        if (cancelled) return;
        const ok = await checkAvailability();
        if (!ok && !cancelled) {
          scheduleRetry();
        }
      }, 3500);
    };

    void (async () => {
      const ok = await checkAvailability();
      if (!ok && !cancelled) {
        scheduleRetry();
      }
    })();

    const onVisibility = () => {
      if (document.visibilityState === 'visible') {
        void checkAvailability();
      }
    };
    document.addEventListener('visibilitychange', onVisibility);

    return () => {
      cancelled = true;
      if (retryTimer) window.clearTimeout(retryTimer);
      document.removeEventListener('visibilitychange', onVisibility);
      wsRef.current?.disconnect();
      wsRef.current = null;
    };
  }, [connectWebSocketOnce]);

  const value: AgentDiagnosticsContextValue = {
    agentServiceAvailable,
    streamRelayRef,
    clearAgentLogs,
    logPanelVisible,
    setLogPanelVisible,
    recheckAgentService,
  };

  return (
    <AgentDiagnosticsContext.Provider value={value}>
      {children}
      <AgentLogPanel
        logs={agentLogs}
        visible={logPanelVisible}
        serviceAvailable={agentServiceAvailable}
        onClear={clearAgentLogs}
        onToggleVisible={() => setLogPanelVisible((v) => !v)}
      />
    </AgentDiagnosticsContext.Provider>
  );
}
