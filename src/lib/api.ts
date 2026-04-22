import { invoke } from '@tauri-apps/api/core';
import type { ChatHistory, Document, AppConfig, Template } from '../types';

export interface SendMessageRequest {
  message: string;
  history_id?: string;
}

export interface SendMessageResponse {
  history_id: string;
  message_id: string;
}

export const api = {
  sendMessage: async (request: SendMessageRequest): Promise<SendMessageResponse> => {
    return await invoke('send_message', { request });
  },

  getHistories: async (): Promise<ChatHistory[]> => {
    return await invoke('get_histories');
  },

  getHistory: async (id: string): Promise<ChatHistory | null> => {
    return await invoke('get_history', { id });
  },

  deleteHistory: async (id: string): Promise<void> => {
    return await invoke('delete_history', { id });
  },

  extractText: async (filePath: string): Promise<string> => {
    return await invoke('extract_text', { filePath });
  },

  getDocuments: async (): Promise<Document[]> => {
    return await invoke('get_documents');
  },

  getConfig: async (): Promise<AppConfig> => {
    return await invoke('get_config');
  },

  updateConfig: async (updates: Record<string, unknown>): Promise<void> => {
    return await invoke('update_config', { updates });
  },

  getTemplates: async (): Promise<Template[]> => {
    return await invoke('get_templates');
  },
};
