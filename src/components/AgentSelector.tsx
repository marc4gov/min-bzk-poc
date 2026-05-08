import React from 'react';
import type { SelectedAgent, AgentMode, AgentInfo, SpecialistId } from '../types/agents';
import { SPECIALIST_IDS } from '../types/agents';

interface AgentSelectorProps {
  selectedAgent: SelectedAgent;
  agentMode: AgentMode;
  onAgentChange: (agent: SelectedAgent) => void;
  onModeChange: (mode: AgentMode) => void;
  /** Van `GET /api/agents` via Tauri; zo blijft UI gelijk aan `agent-service`/registry. */
  specialistsFromService?: AgentInfo[] | null;
  disabled?: boolean;
}

/** Fallback wanneer de API nog niet geladen is (kort gehouden; volledige copy komt uit de service). */
const FALLBACK_META: Record<
  SpecialistId,
  { name: string; description: string }
> = {
  code: { name: 'Code', description: 'Programmeren, debuggen' },
  schrijf: { name: 'Schrijf', description: 'Nederlandse teksten' },
  tools: { name: 'Tools', description: 'Bestanden, web, shell' },
  research: { name: 'Research', description: 'Actueel web' },
  translate: { name: 'Vertalen', description: 'Taal & tone' },
  analyze: { name: 'Analyse', description: 'Bestanden & mappen' },
  docs: { name: 'Docs', description: 'README & technische docs' },
  brainstorm: { name: 'Brainstorm', description: 'Ideeën & concepten' },
  data: { name: 'Data', description: 'CSV / JSON / logs' },
  general: { name: 'Algemeen', description: 'Zonder tools' },
};

export const AgentSelector: React.FC<AgentSelectorProps> = ({
  selectedAgent,
  agentMode,
  onAgentChange,
  onModeChange,
  specialistsFromService,
  disabled = false,
}) => {
  /** Mesh staat alleen voor overzicht; chat routeert naar `ROUTING_IDS`/`SPECIALISTS`. */
  const selectableChat = (a: AgentInfo) =>
    (a.kind === undefined || a.kind !== 'mesh') && a.selectable !== false;

  const specialistRows =
    specialistsFromService && specialistsFromService.length > 0
      ? specialistsFromService
          .filter((a) => a.id !== 'orchestrator' && selectableChat(a))
          .map((a) => ({
            value: a.id as SpecialistId,
            label: a.name,
            description: a.description,
          }))
      : SPECIALIST_IDS.map((id) => ({
          value: id,
          label: FALLBACK_META[id].name,
          description: FALLBACK_META[id].description,
        }));

  const meshRows =
    specialistsFromService?.filter((a) => a.kind === 'mesh').map((a) => ({
      key: a.id,
      label: a.name,
      description: a.description,
    })) ?? [];

  const agents = [
    {
      value: 'auto' as const,
      label: 'Auto',
      description: 'Orchestrator kiest specialist (JSON-routing uit registry)',
    },
    ...specialistRows,
  ];

  return (
    <div className="flex items-center space-x-2 flex-wrap gap-y-2">
      <div className="flex items-center space-x-1 bg-zinc-800 rounded-lg p-1">
        <button
          type="button"
          onClick={() => onModeChange('single')}
          disabled={disabled}
          className={`px-3 py-1 rounded text-sm transition-colors ${
            agentMode === 'single'
              ? 'bg-indigo-600 text-white'
              : 'text-zinc-400 hover:text-white'
          } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
        >
          Single
        </button>
        <button
          type="button"
          onClick={() => onModeChange('multi')}
          disabled={disabled}
          className={`px-3 py-1 rounded text-sm transition-colors ${
            agentMode === 'multi'
              ? 'bg-indigo-600 text-white'
              : 'text-zinc-400 hover:text-white'
          } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
        >
          Multi
        </button>
      </div>

      {agentMode === 'multi' && (
        <select
          value={selectedAgent}
          onChange={(e) => onAgentChange(e.target.value as SelectedAgent)}
          disabled={disabled}
          className={`bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-1 text-sm text-white focus:outline-none focus:ring-2 focus:ring-indigo-500 max-w-[min(100vw-2rem,24rem)] ${
            disabled ? 'opacity-50 cursor-not-allowed' : ''
          }`}
        >
          {agents.map((agent) => (
            <option key={agent.value} value={agent.value} title={agent.description}>
              {agent.label}
            </option>
          ))}
          {meshRows.length > 0 && (
            <optgroup label="Mesh (ractor; via Mesh-demo, niet via chat)">
              {meshRows.map((m) => (
                <option key={m.key} value={m.key} disabled title={m.description}>
                  {m.label}
                </option>
              ))}
            </optgroup>
          )}
        </select>
      )}
    </div>
  );
};
