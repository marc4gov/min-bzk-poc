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

export interface ModelInfo {
  id: string;
  name: string;
  url: string;
  size_mb: number;
  description: string;
}

export interface DownloadProgress {
  model_id: string;
  downloaded_bytes: number;
  total_bytes: number;
  percentage: number;
}

// Timeout helper - aborts promise if it takes too long
const withTimeout = <T>(promise: Promise<T>, ms: number, errorMessage: string): Promise<T> => {
  return Promise.race([
    promise,
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new Error(errorMessage)), ms)
    ),
  ]);
};

export const api = {
  sendMessage: async (request: SendMessageRequest): Promise<SendMessageResponse> => {
    // 3 minute timeout for LLM generation
    return withTimeout(
      invoke('send_message', { request }),
      180000,
      'De assistent reageert niet (time-out). Controleer of llama-cli correct is geïnstalleerd.'
    );
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

  deleteAllHistories: async (): Promise<number> => {
    return await invoke('delete_all_histories');
  },

  extractText: async (filePath: string): Promise<string> => {
    return await invoke('extract_text', { filePath });
  },

  uploadDocumentForImprovement: async (filePath: string, instructions: string): Promise<{
    filename: string;
    content: string;
    file_type: string;
    preview: string;
  }> => {
    return await invoke('upload_document_for_improvement', {
      request: { file_path: filePath, instructions },
    });
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

  // Model management
  getAvailableModels: async (): Promise<ModelInfo[]> => {
    return await invoke('get_available_models');
  },

  getDownloadedModels: async (): Promise<string[]> => {
    return await invoke('get_downloaded_models');
  },

  downloadModel: async (modelId: string, onProgress?: (progress: DownloadProgress) => void): Promise<string> => {
    const unlisten = onProgress ? await import('@tauri-apps/api/event').then(m =>
      m.listen('model-download-progress', (event) => onProgress(event.payload as DownloadProgress))
    ) : Promise.resolve(() => {});

    try {
      const filename = await invoke('download_model', { modelId });
      return filename as string;
    } finally {
      if (onProgress) {
        (await unlisten)();
      }
    }
  },

  deleteModel: async (modelId: string): Promise<void> => {
    return await invoke('delete_model', { modelId });
  },
};
