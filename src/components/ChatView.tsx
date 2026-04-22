import React, { useState, useRef, useEffect, useImperativeHandle, forwardRef } from 'react';
import { Button } from './ui/Button';
import { Textarea } from './ui/Textarea';
import type { ChatMessage, ChatHistory } from '../types';
import { api } from '../lib/api';
import { marked } from 'marked';

interface ChatViewProps {
  historyId: string | null;
  onHistoryChange: (id: string) => void;
}

export interface ChatViewRef {
  insertPrompt: (prompt: string) => void;
}

export const ChatView = forwardRef<ChatViewRef, ChatViewProps>(({ historyId, onHistoryChange }, ref) => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [currentHistory, setCurrentHistory] = useState<ChatHistory | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useImperativeHandle(ref, () => ({
    insertPrompt: (prompt: string) => {
      setInput(prompt);
      textareaRef.current?.focus();
    },
  }));

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
      setCurrentHistory(null);
    }
  }, [historyId]);

  const loadHistory = async (id: string) => {
    try {
      const history = await api.getHistory(id);
      if (history) {
        setMessages(history.messages);
        setCurrentHistory(history);
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
    const currentInput = input;
    setInput('');
    setIsLoading(true);

    try {
      const response = await api.sendMessage({
        message: currentInput,
        history_id: historyId || undefined,
      });

      if (response.history_id !== historyId) {
        onHistoryChange(response.history_id);
      }

      await loadHistory(response.history_id);
    } catch (error) {
      console.error('Failed to send message:', error);
      setMessages((prev) => [...prev.slice(0, -1)]);
      setInput(currentInput);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a]">
      {currentHistory && (
        <div className="px-4 py-2 border-b border-zinc-800 bg-[#1a1a1a]">
          <h2 className="text-sm font-medium text-zinc-300">{currentHistory.title}</h2>
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-zinc-500">
            <svg className="w-16 h-16 mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
            </svg>
            <p className="text-lg">Start een gesprek...</p>
            <p className="text-sm mt-2">Typ een bericht hieronder of gebruik een template</p>
          </div>
        ) : (
          messages.map((message) => (
            <div
              key={message.id}
              className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-[80%] rounded-2xl px-4 py-2 ${
                  message.role === 'user'
                    ? 'bg-indigo-600 text-white rounded-br-sm'
                    : 'bg-zinc-800 text-white rounded-bl-sm'
                }`}
              >
                {message.role === 'assistant' ? (
                  <div
                    className="prose prose-invert prose-p:my-1 prose-headings:mt-2 prose-headings:mb-1 max-w-none"
                    dangerouslySetInnerHTML={{ __html: marked.parse(message.content) as string }}
                  />
                ) : (
                  <p className="whitespace-pre-wrap break-words">{message.content}</p>
                )}
                <div className={`text-xs opacity-50 mt-1 ${message.role === 'user' ? 'text-right' : 'text-left'}`}>
                  {new Date(message.timestamp).toLocaleTimeString('nl-NL', {
                    hour: '2-digit',
                    minute: '2-digit',
                  })}
                </div>
              </div>
            </div>
          ))
        )}
        {isLoading && (
          <div className="flex justify-start">
            <div className="bg-zinc-800 rounded-2xl rounded-bl-sm px-4 py-3">
              <div className="flex space-x-1">
                <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" />
                <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" style={{ animationDelay: '0.1s' }} />
                <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }} />
              </div>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <form onSubmit={handleSubmit} className="p-4 border-t border-zinc-800 bg-[#1a1a1a]">
        <div className="flex items-end space-x-2">
          <Textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Typ je bericht... (Enter om te versturen, Shift+Enter voor nieuwe regel)"
            className="flex-1 min-h-[60px] max-h-[200px] resize-none"
            disabled={isLoading}
          />
          <Button type="submit" disabled={!input.trim() || isLoading} className="px-6">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
            </svg>
          </Button>
        </div>
      </form>
    </div>
  );
});
