import React, { useState, useEffect, useCallback } from 'react';
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
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [isDeletingAll, setIsDeletingAll] = useState(false);

  const loadHistories = useCallback(async () => {
    try {
      const data = await api.getHistories();
      setHistories(data);
    } catch (error) {
      console.error('Failed to load histories:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadHistories();
  }, [loadHistories]);

  const handleDelete = async (id: string, title: string) => {
    if (!confirm(`"${title}" verwijderen?`)) return;

    setDeletingId(id);

    // Optimistically remove from UI
    setHistories(prev => prev.filter(h => h.id !== id));

    try {
      await api.deleteHistory(id);
      if (currentHistoryId === id) {
        onNewChat();
      }
    } catch (error) {
      console.error('Delete error:', error);
      alert('Kon gesprek niet verwijderen');
      // Reload on error to restore the item
      await loadHistories();
    } finally {
      setDeletingId(null);
    }
  };

  const handleDeleteAll = async () => {
    if (histories.length === 0) return;
    if (!confirm(`Weet je zeker dat je alle ${histories.length} gesprekken wilt verwijderen?`)) return;

    setIsDeletingAll(true);

    // Optimistically clear UI
    const previousHistories = histories;
    setHistories([]);

    try {
      const count = await api.deleteAllHistories();
      onNewChat();
      alert(`${count} gesprekken verwijderd`);
      // Reload to confirm
      await loadHistories();
    } catch (error) {
      console.error('Delete all error:', error);
      alert('Kon niet alle gesprekken verwijderen');
      // Restore on error
      setHistories(previousHistories);
    } finally {
      setIsDeletingAll(false);
    }
  };

  return (
    <div className="w-64 bg-[#1a1a1a] border-r border-zinc-800 flex flex-col">
      <div className="p-4 border-b border-zinc-800">
        <Button onClick={onNewChat} className="w-full mb-2">
          + Nieuw gesprek
        </Button>
        {histories.length > 0 && (
          <button
            onClick={handleDeleteAll}
            disabled={isDeletingAll}
            className="w-full py-2 px-3 text-xs text-red-400 hover:text-red-300 hover:bg-red-900/20 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isDeletingAll ? 'Verwijderen...' : '🗑️ Verwijder alles'}
          </button>
        )}
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
                className={`p-3 rounded-lg transition-colors ${
                  currentHistoryId === history.id
                    ? 'bg-indigo-600 text-white'
                    : 'hover:bg-zinc-800 text-zinc-300'
                } ${deletingId === history.id ? 'opacity-50' : ''}`}
              >
                <div className="flex items-center gap-2">
                  <span
                    className="flex-1 truncate text-sm cursor-pointer"
                    onClick={() => onSelectHistory(history.id)}
                  >
                    {history.title}
                  </span>
                  <button
                    onClick={() => handleDelete(history.id, history.title)}
                    disabled={deletingId === history.id}
                    className="p-1 text-zinc-500 hover:text-red-400 hover:bg-zinc-700/50 rounded transition-colors"
                    type="button"
                    title="Verwijderen"
                  >
                    {deletingId === history.id ? '...' : '🗑️'}
                  </button>
                </div>
                <div className="text-xs opacity-60 mt-1">
                  {new Date(history.updated_at).toLocaleDateString('nl-NL', {
                    day: 'numeric',
                    month: 'short',
                    year: 'numeric'
                  })}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
