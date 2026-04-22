import React, { useState, useRef, useEffect } from 'react';
import { Button } from './ui/Button';
import { Textarea } from './ui/Textarea';
import type { ChatMessage } from '../types';
import { api } from '../lib/api';
import { marked } from 'marked';

interface ChatViewProps {
  historyId: string | null;
  onHistoryChange: (id: string) => void;
}

export const ChatView: React.FC<ChatViewProps> = ({ historyId, onHistoryChange }) => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    if (historyId) {
      loadHistory(historyId);
    } else {
      setMessages([]);
    }
  }, [historyId]);

  const loadHistory = async (id: string) => {
    try {
      const history = await api.getHistory(id);
      if (history) {
        setMessages(history.messages);
      }
    } catch (error) {
      console.error('Failed to load history:', error);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: input,
      timestamp: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    try {
      const response = await api.sendMessage({
        message: input,
        history_id: historyId || undefined,
      });

      if (response.history_id !== historyId) {
        onHistoryChange(response.history_id);
      }

      await loadHistory(response.history_id);
    } catch (error) {
      console.error('Failed to send message:', error);
      setMessages((prev) => [...prev.slice(0, -1)]);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a]">
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="flex items-center justify-center h-full text-zinc-500">
            <p className="text-lg">Start een gesprek...</p>
          </div>
        )}
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[80%] rounded-lg px-4 py-2 ${
                message.role === 'user'
                  ? 'bg-indigo-600 text-white'
                  : 'bg-zinc-800 text-white'
              }`}
            >
              {message.role === 'assistant' ? (
                <div
                  className="prose prose-invert max-w-none"
                  dangerouslySetInnerHTML={{ __html: marked.parse(message.content) as string }}
                />
              ) : (
                <p className="whitespace-pre-wrap">{message.content}</p>
              )}
            </div>
          </div>
        ))}
        {isLoading && (
          <div className="flex justify-start">
            <div className="bg-zinc-800 rounded-lg px-4 py-2">
              <div className="flex space-x-2">
                <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" />
                <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" style={{ animationDelay: '0.1s' }} />
                <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }} />
              </div>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <form onSubmit={handleSubmit} className="p-4 border-t border-zinc-800">
        <div className="flex space-x-2">
          <Textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Typ je bericht..."
            className="flex-1 min-h-[60px] max-h-[200px]"
            disabled={isLoading}
          />
          <Button type="submit" disabled={!input.trim() || isLoading}>
            Verstuur
          </Button>
        </div>
      </form>
    </div>
  );
};
