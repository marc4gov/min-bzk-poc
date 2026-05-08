import React, { useMemo } from 'react';
import type { AgentLogEntry, AgentEventType } from '../types/agents';

interface AgentLogPanelProps {
  logs: AgentLogEntry[];
  visible: boolean;
  serviceAvailable: boolean;
  onClear: () => void;
  onToggleVisible: () => void;
}

const EVENT_ICONS: Record<AgentEventType, string> = {
  connected: '🔌',
  orchestrator_start: '🎯',
  orchestrator_decision: '🔀',
  agent_start: '▶️',
  agent_thinking: '🤔',
  agent_tool_use: '🔧',
  agent_tool_result: '📋',
  agent_complete: '✅',
  agent_error: '❌',
  streaming: '📝',
  mesh_step: '🕸️',
};

const EVENT_COLORS: Record<AgentEventType, string> = {
  connected: 'text-zinc-400',
  orchestrator_start: 'text-purple-400',
  orchestrator_decision: 'text-purple-300',
  agent_start: 'text-blue-400',
  agent_thinking: 'text-yellow-400',
  agent_tool_use: 'text-orange-400',
  agent_tool_result: 'text-green-400',
  agent_complete: 'text-emerald-400',
  agent_error: 'text-red-400',
  streaming: 'text-zinc-300',
  mesh_step: 'text-cyan-400',
};

const EVENT_LABELS: Record<AgentEventType, string> = {
  connected: 'Verbonden',
  orchestrator_start: 'Orchestrator',
  orchestrator_decision: 'Routing',
  agent_start: 'Gestart',
  agent_thinking: 'Denkt',
  agent_tool_use: 'Tool',
  agent_tool_result: 'Resultaat',
  agent_complete: 'Klaar',
  agent_error: 'Fout',
  streaming: 'Stream',
  mesh_step: 'Mesh',
};

export const AgentLogPanel: React.FC<AgentLogPanelProps> = ({
  logs,
  visible,
  serviceAvailable,
  onClear,
  onToggleVisible,
}) => {
  const formatTime = (timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString('nl-NL', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  const formatDetails = (entry: AgentLogEntry) => {
    // Model informatie eerst tonen
    if (entry.details?.model) {
      return `🤖 Model: ${entry.details.model as string}`;
    }
    if (entry.tool_result) {
      const result = entry.tool_result as Record<string, unknown>;
      if (result.content) {
        return `📄 ${JSON.stringify(result.content).substring(0, 100)}...`;
      }
      if (result.stdout) {
        return `📤 ${result.stdout}`;
      }
      if (result.entries) {
        const entries = result.entries as Array<{ name: string; type: string }>;
        return `📁 ${entries.map(e => e.name).join(', ')}`;
      }
      return JSON.stringify(result).substring(0, 100);
    }
    if (entry.details?.content_preview) {
      return `"${(entry.details.content_preview as string).substring(0, 80)}..."`;
    }
    if (entry.details?.args) {
      return `Args: ${JSON.stringify(entry.details.args)}`;
    }
    if (entry.details?.reasoning) {
      return `💭 ${entry.details.reasoning as string}`;
    }
    return null;
  };

  const groupedLogs = useMemo(() => {
    const groups: Record<string, AgentLogEntry[]> = {};
    logs.forEach((log) => {
      const key = log.agent || 'system';
      if (!groups[key]) groups[key] = [];
      groups[key].push(log);
    });
    return groups;
  }, [logs]);

  if (!visible) {
    return (
      <button
        onClick={onToggleVisible}
        className="fixed bottom-4 right-4 z-50 bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-lg shadow-lg flex items-center space-x-2"
      >
        <span className="text-lg">📋</span>
        <span>Agent Logs</span>
        {logs.length > 0 && (
          <span className="bg-indigo-500 px-2 py-0.5 rounded-full text-xs">
            {logs.length}
          </span>
        )}
      </button>
    );
  }

  return (
    <div className="fixed bottom-0 right-0 w-[500px] h-[400px] bg-[#0a0a0a] border-t border-l border-zinc-700 rounded-tl-lg shadow-2xl flex flex-col z-50">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-zinc-700 bg-[#1a1a1a] rounded-tl-lg">
        <div className="flex items-center space-x-2">
          <span className="text-lg">📋</span>
          <h3 className="font-medium text-white">Agent Communicatie</h3>
          {!serviceAvailable && (
            <span className="bg-red-900/50 text-red-300 px-2 py-0.5 rounded-full text-xs">
              Offline
            </span>
          )}
          {serviceAvailable && (
            <span className="bg-green-900/50 text-green-300 px-2 py-0.5 rounded-full text-xs">
              Online
            </span>
          )}
          {logs.length > 0 && (
            <span className="bg-zinc-700 px-2 py-0.5 rounded-full text-xs text-zinc-300">
              {logs.length}
            </span>
          )}
        </div>
        <div className="flex items-center space-x-2">
          <button
            onClick={onClear}
            className="text-xs text-zinc-400 hover:text-zinc-200 px-2 py-1 rounded hover:bg-zinc-700"
          >
            Wissen
          </button>
          <button
            onClick={onToggleVisible}
            className="text-zinc-400 hover:text-white p-1 rounded hover:bg-zinc-700"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </button>
        </div>
      </div>

      {/* Logs */}
      <div className="flex-1 overflow-y-auto p-3 space-y-1">
        {!serviceAvailable ? (
          <div className="flex flex-col items-center justify-center h-full text-zinc-500">
            <svg className="w-12 h-12 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
            <p className="text-sm">Agent service niet beschikbaar</p>
            <p className="text-xs mt-1">Start de agent service om logs te zien</p>
          </div>
        ) : logs.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-zinc-500">
            <svg className="w-12 h-12 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
            <p className="text-sm">Nog geen activiteit</p>
            <p className="text-xs mt-1">Stuur een bericht om te beginnen</p>
          </div>
        ) : (
          Object.entries(groupedLogs).map(([agent, agentLogs]) => (
            <div key={agent} className="mb-3">
              <div className="flex items-center space-x-2 mb-1">
                <span className="text-xs font-medium text-zinc-400 uppercase tracking-wide">
                  {agent === 'system' ? 'Systeem' : agent}
                </span>
                <div className="flex-1 h-px bg-zinc-800" />
              </div>
              <div className="space-y-1 ml-2">
                {agentLogs.map((log) => (
                  <div key={log.id} className="text-xs">
                    <div className="flex items-start space-x-2">
                      <span className={EVENT_COLORS[log.type]}>
                        {EVENT_ICONS[log.type]}
                      </span>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center space-x-2">
                          <span className={EVENT_COLORS[log.type]}>
                            {EVENT_LABELS[log.type]}
                          </span>
                          <span className="text-zinc-500">{formatTime(log.timestamp)}</span>
                        </div>
                        {log.message && (
                          <p className="text-zinc-300 mt-0.5">{log.message}</p>
                        )}
                        {formatDetails(log) && (
                          <pre className="text-zinc-400 mt-0.5 whitespace-pre-wrap break-words bg-zinc-900/50 px-2 py-1 rounded">
                            {formatDetails(log)}
                          </pre>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Status bar */}
      <div className="px-4 py-1 border-t border-zinc-700 bg-[#1a1a1a]">
        <div className="flex items-center justify-between text-xs text-zinc-500">
          <span>{logs.length} events</span>
          <span>Live updates</span>
        </div>
      </div>
    </div>
  );
};
