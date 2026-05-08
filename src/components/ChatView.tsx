import React, { useState, useRef, useEffect, useImperativeHandle, forwardRef } from 'react';
import { Button } from './ui/Button';
import { Textarea } from './ui/Textarea';
import { AgentSelector } from './AgentSelector';
import { DocumentUpload } from './DocumentUpload';
import type { ChatMessage, ChatHistory } from '../types';
import type { AgentMode, SelectedAgent, AgentInfo, AgentStreamingContent } from '../types/agents';
import { useAgentDiagnostics } from '../AgentDiagnosticsProvider';
import { api } from '../lib/api';
import { agentApi } from '../lib/agent-api';
import {
  getStoredModel,
  persistModel,
  type ModelPrefsScope,
} from '../lib/agent-model-prefs';
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
  const [agentMode, setAgentMode] = useState<AgentMode>('single');
  const [selectedAgent, setSelectedAgent] = useState<SelectedAgent>('auto');
  const { agentServiceAvailable, streamRelayRef } = useAgentDiagnostics();
  /** Specialist-metadata van `GET /api/agents` — gelijk aan `agent-service/src/registry.rs`. */
  const [serviceSpecialists, setServiceSpecialists] = useState<AgentInfo[] | null>(null);
  const [streamingMessageId, setStreamingMessageId] = useState<string | null>(null);
  const streamingAssistantIdRef = useRef<string | null>(null);
  /** Ollama-tags van `ollama_models` voor modeldropdown (Single + Multi). */
  const [ollamaModelNames, setOllamaModelNames] = useState<string[]>([]);
  const [selectedOllamaModel, setSelectedOllamaModel] = useState('');
  /** Document upload UI state */
  const [showDocumentUpload, setShowDocumentUpload] = useState(false);

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

  useEffect(() => {
    if (!agentServiceAvailable) {
      setServiceSpecialists(null);
      return;
    }
    let cancelled = false;
    agentApi
      .getAgents()
      .then((list) => {
        if (!cancelled) setServiceSpecialists(list);
      })
      .catch(() => {
        if (!cancelled) setServiceSpecialists(null);
      });
    return () => {
      cancelled = true;
    };
  }, [agentServiceAvailable]);

  useEffect(() => {
    if (!agentServiceAvailable) return;
    let cancelled = false;
    agentApi
      .listOllamaModels()
      .then((r) => {
        if (
          cancelled ||
          !(r && typeof r === 'object' && 'ok' in r && (r as { ok: boolean }).ok)
        )
          return;
        const rm = r as { models?: { name: string }[] };
        if (Array.isArray(rm.models)) {
          setOllamaModelNames(rm.models.map((m) => m.name).filter(Boolean));
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [agentServiceAvailable]);

  /** Streamchunks van AgentDiagnosticsProvider → actieve assistent-bericht. */
  useEffect(() => {
    const handler = (stream: AgentStreamingContent) => {
      const targetId = streamingAssistantIdRef.current;
      if (!targetId) return;
      setMessages((prev) =>
        prev.map((m) =>
          m.id === targetId
            ? { ...m, content: stream.content, timestamp: m.timestamp }
            : m
        )
      );
    };
    streamRelayRef.current = handler;
    return () => {
      streamRelayRef.current = null;
    };
  }, [streamRelayRef]);

  const modelPrefsScope: ModelPrefsScope =
    agentMode === 'single' ? 'single' : selectedAgent;

  useEffect(() => {
    if (!ollamaModelNames.length) return;
    const stored = getStoredModel(modelPrefsScope);
    if (stored && ollamaModelNames.includes(stored)) {
      setSelectedOllamaModel(stored);
      return;
    }
    setSelectedOllamaModel((cur) =>
      cur && ollamaModelNames.includes(cur) ? cur : ollamaModelNames[0] ?? ''
    );
  }, [modelPrefsScope, ollamaModelNames]);

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
      if (agentServiceAvailable) {
        const assistantId = `${Date.now()}-asst`;
        streamingAssistantIdRef.current = assistantId;
        setStreamingMessageId(assistantId);

        const assistantPlaceholder: ChatMessage = {
          id: assistantId,
          role: 'assistant',
          content: '',
          timestamp: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, assistantPlaceholder]);

        try {
          const agentResponse = await agentApi.sendMessage({
            message: currentInput,
            agent_mode: agentMode,
            selected_agent: selectedAgent,
            ...(selectedOllamaModel.trim()
              ? { model: selectedOllamaModel.trim() }
              : {}),
          });

          setMessages((prev) =>
            prev.map((m) =>
              m.id === assistantId
                ? { ...m, content: agentResponse.content, timestamp: new Date().toISOString() }
                : m
            )
          );

          if (agentResponse.agents_used.length > 0 && import.meta.env.DEV) {
            console.log('Agents used:', agentResponse.agents_used, 'model:', agentResponse.model_used);
          }
        } catch (error) {
          console.error('Failed to send message (agent service):', error);
          const errText =
            error instanceof Error ? error.message : 'Er ging iets mis. Probeer het opnieuw.';
          setMessages((prev) =>
            prev.map((m) =>
              m.id === assistantId ? { ...m, content: `Fout: ${errText}` } : m
            )
          );
        } finally {
          streamingAssistantIdRef.current = null;
          setStreamingMessageId(null);
        }
      } else {
        const response = await api.sendMessage({
          message: currentInput,
          history_id: historyId || undefined,
        });

        if (response.history_id !== historyId) {
          onHistoryChange(response.history_id);
        }

        await loadHistory(response.history_id);
      }
    } catch (error) {
      console.error('Failed to send message:', error);

      const errText =
        error instanceof Error ? error.message : 'Er ging iets mis. Probeer het opnieuw.';
      const errorMessage: ChatMessage = {
        id: Date.now().toString() + '-error',
        role: 'assistant',
        content: errText,
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, errorMessage]);
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

  const handleDocumentUpload = async (filename: string, content: string, instructions: string) => {
    setShowDocumentUpload(false);

    // Add user message with document info
    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: `📎 Document: "${filename}"\n\nInstructies: ${instructions}`,
      timestamp: new Date().toISOString(),
    };
    setMessages((prev) => [...prev, userMessage]);
    setIsLoading(true);

    try {
      const assistantId = `${Date.now()}-asst`;
      streamingAssistantIdRef.current = assistantId;
      setStreamingMessageId(assistantId);

      const assistantPlaceholder: ChatMessage = {
        id: assistantId,
        role: 'assistant',
        content: '',
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, assistantPlaceholder]);

      // Use the mesh improve document endpoint
      const result = await agentApi.improveDocument(
        content,
        filename,
        instructions,
        selectedOllamaModel.trim() || undefined
      );

      if (result.ok && result.result) {
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantId
              ? { ...m, content: result.result!, timestamp: new Date().toISOString() }
              : m
          )
        );
      } else {
        const errorMessage: ChatMessage = {
          id: Date.now().toString() + '-error',
          role: 'assistant',
          content: `Fout bij documentverbetering: ${result.error || 'Onbekende fout'}`,
          timestamp: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, errorMessage]);
      }
    } catch (error) {
      console.error('Document improvement failed:', error);
      const errText = error instanceof Error ? error.message : 'Er ging iets mis.';
      const errorMessage: ChatMessage = {
        id: Date.now().toString() + '-error',
        role: 'assistant',
        content: `Fout bij documentverbetering: ${errText}`,
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
      streamingAssistantIdRef.current = null;
      setStreamingMessageId(null);
    }
  };

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a]">
      {currentHistory && (
        <div className="px-4 py-2 border-b border-zinc-800 bg-[#1a1a1a]">
          <h2 className="text-sm font-medium text-zinc-300">{currentHistory.title}</h2>
        </div>
      )}
      {agentServiceAvailable && (
        <div className="px-4 py-2 border-b border-zinc-800 bg-[#1a1a1a]">
          <AgentSelector
            selectedAgent={selectedAgent}
            agentMode={agentMode}
            onAgentChange={setSelectedAgent}
            onModeChange={setAgentMode}
            specialistsFromService={serviceSpecialists}
            disabled={isLoading}
          />
          {ollamaModelNames.length > 0 && (
            <label className="flex items-center gap-2 text-sm text-zinc-400 ml-2">
              <span className="whitespace-nowrap shrink-0">Model</span>
              <select
                value={selectedOllamaModel}
                onChange={(e) => {
                  const v = e.target.value;
                  setSelectedOllamaModel(v);
                  persistModel(modelPrefsScope, v);
                }}
                disabled={isLoading}
                className={`bg-zinc-900 border border-zinc-700 rounded-lg px-2 py-1 text-xs text-white max-w-[min(100vw-8rem,14rem)] ${
                  isLoading ? 'opacity-50 cursor-not-allowed' : ''
                }`}
                title={
                  agentMode === 'single'
                    ? 'Ollama-model voor Single-modus (orchestrator); wordt lokaal voor Single opgeslagen'
                    : 'Ollama-tag per bericht; wordt per gekozen specialist onthouden op dit apparaat'
                }
              >
                {ollamaModelNames.map((name) => (
                  <option key={name} value={name}>
                    {name}
                  </option>
                ))}
              </select>
            </label>
          )}
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
                  message.content === '' ? (
                    <div className="flex items-center gap-1 py-1 text-zinc-400 text-sm" aria-live="polite">
                      <span className="inline-block w-2 h-2 bg-zinc-500 rounded-full animate-pulse" />
                      <span>Antwoord wordt gestreamd…</span>
                    </div>
                  ) : (
                    <div
                      className="prose prose-invert prose-p:my-1 prose-headings:mt-2 prose-headings:mb-1 max-w-none"
                      dangerouslySetInnerHTML={{ __html: marked.parse(message.content) as string }}
                    />
                  )
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
        {isLoading && !streamingMessageId && (
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
        {/* Document upload section */}
        {showDocumentUpload && (
          <div className="mb-4">
            {agentServiceAvailable ? (
              <DocumentUpload
                onUploadStart={handleDocumentUpload}
                disabled={isLoading}
              />
            ) : (
              <div className="border border-zinc-700 rounded-lg p-4 bg-[#1a1a1a]">
                <p className="text-sm text-zinc-400">
                  ⚠️ Document verbetering vereist de agent service. Start de agent service om deze functie te gebruiken.
                </p>
                <button
                  type="button"
                  onClick={() => setShowDocumentUpload(false)}
                  className="mt-2 text-xs text-zinc-500 hover:text-zinc-400"
                >
                  Sluiten
                </button>
              </div>
            )}
          </div>
        )}

        {/* Input area */}
        {!showDocumentUpload && (
          <div className="flex items-end space-x-2">
            <Button
              type="button"
              onClick={() => setShowDocumentUpload(true)}
              disabled={isLoading}
              variant="secondary"
              className="px-3"
              title="Document uploaden"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
              </svg>
            </Button>
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
        )}
      </form>
    </div>
  );
});
