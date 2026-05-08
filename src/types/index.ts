export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}

export interface ChatHistory {
  id: string;
  title: string;
  messages: ChatMessage[];
  created_at: string;
  updated_at: string;
}

export interface Document {
  id: string;
  filename: string;
  file_path: string;
  content_preview: string;
  extracted_text?: string;
  created_at: string;
}

export interface Template {
  id: string;
  name: string;
  description: string;
  prompt_template: string;
  is_builtin: boolean;
}

export interface AppConfig {
  model_path: string;
  temperature: number;
  top_p: number;
  max_tokens: number;
  theme: string;
}

export * from './agents';
