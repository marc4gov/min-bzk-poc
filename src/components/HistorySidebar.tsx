import React, { useState, useEffect } from 'react';
import { Button } from './ui/Button';
import type { ChatHistory } from '../types';
import { api } from '../lib/api';

interface HistorySidebarProps {
  currentHistoryId: string | null;
  onSelectHistory: (id: string) => void;
  onNewChat: () => void;
}

export const HistorySidebar: React.FC<HistorySidebarProps> = ({
  currentHistoryId,
  onSelectHistory,
  onNewChat,
}) => {
  const [histories, setHistories] = useState<ChatHistory[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadHistories();
  }, []);

  const loadHistories = async () => {
    try {
      const data = await api.getHistories();
      setHistories(data);
    } catch (error) {
      console.error('Failed to load histories:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!confirm('Gesprek verwijderen?')) return;

    try {
      await api.deleteHistory(id);
      await loadHistories();
      if (currentHistoryId === id) {
        onNewChat();
      }
    } catch (error) {
      console.error('Failed to delete history:', error);
    }
  };

  return (
    <div className="w-64 bg-[#1a1a1a] border-r border-zinc-800 flex flex-col">
      <div className="p-4 border-b border-zinc-800">
        <Button onClick={onNewChat} className="w-full">
          + Nieuw gesprek
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-2">
        {isLoading ? (
          <div className="text-zinc-500 text-center py-4">Laden...</div>
        ) : histories.length === 0 ? (
          <div className="text-zinc-500 text-center py-4">Nog geen gesprekken</div>
        ) : (
          <div className="space-y-1">
            {histories.map((history) => (
              <div
                key={history.id}
                onClick={() => onSelectHistory(history.id)}
                className={`p-3 rounded-lg cursor-pointer transition-colors ${
                  currentHistoryId === history.id
                    ? 'bg-indigo-600 text-white'
                    : 'hover:bg-zinc-800 text-zinc-300'
                } group`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1 truncate text-sm">{history.title}</div>
                  <button
                    onClick={(e) => handleDelete(history.id, e)}
                    className="opacity-0 group-hover:opacity-100 text-zinc-400 hover:text-red-400 transition-opacity"
                  >
                    ×
                  </button>
                </div>
                <div className="text-xs opacity-60 mt-1">
                  {new Date(history.updated_at).toLocaleDateString('nl-NL')}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
