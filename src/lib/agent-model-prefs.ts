/** localStorage: Ollama-modelkeuze (Single-modus vs per specialist in Multi). */

import type { SelectedAgent } from '../types/agents';

const STORAGE_KEY = 'localAssistant.ollamaModelsPerAgent.v1';

/** `single` = Single-modus (orchestrator); anders de gekozen specialist in Multi. */
export type ModelPrefsScope = 'single' | SelectedAgent;

export type StoredModelPrefs = Partial<Record<ModelPrefsScope, string>>;

export function loadModelPrefs(): StoredModelPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as StoredModelPrefs;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

export function persistModel(scope: ModelPrefsScope, modelName: string): void {
  const prefs = loadModelPrefs();
  prefs[scope] = modelName.trim();
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
  } catch {
    /* ignore quota */
  }
}

export function getStoredModel(scope: ModelPrefsScope): string | undefined {
  const m = loadModelPrefs()[scope];
  return typeof m === 'string' && m.trim() !== '' ? m.trim() : undefined;
}
