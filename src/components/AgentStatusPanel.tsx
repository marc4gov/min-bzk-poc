import React from 'react';
import type { AgentStatus } from '../types/agents';

interface AgentStatusPanelProps {
  agents: AgentStatus[];
  visible?: boolean;
}

export const AgentStatusPanel: React.FC<AgentStatusPanelProps> = ({
  agents,
  visible = true,
}) => {
  if (!visible || agents.length === 0) {
    return null;
  }

  const getStateColor = (state: AgentStatus['state']) => {
    switch (state) {
      case 'thinking':
        return 'bg-yellow-500';
      case 'tool-using':
        return 'bg-blue-500';
      case 'complete':
        return 'bg-green-500';
      default:
        return 'bg-zinc-500';
    }
  };

  const getStateLabel = (state: AgentStatus['state']) => {
    switch (state) {
      case 'thinking':
        return 'Denkt...';
      case 'tool-using':
        return 'Werkt...';
      case 'complete':
        return 'Klaar';
      default:
        return 'Inactief';
    }
  };

  return (
    <div className="px-4 py-2 border-b border-zinc-800 bg-[#1a1a1a]">
      <div className="flex items-center space-x-4 text-sm">
        <span className="text-zinc-400">Agenten:</span>
        {agents.map((agent) => (
          <div key={agent.id} className="flex items-center space-x-2">
            <div className={`w-2 h-2 rounded-full ${getStateColor(agent.state)}`} />
            <span className="text-zinc-300">{agent.name}</span>
            <span className="text-zinc-500">({getStateLabel(agent.state)})</span>
          </div>
        ))}
      </div>
    </div>
  );
};
