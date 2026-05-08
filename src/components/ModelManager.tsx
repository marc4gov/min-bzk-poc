import { useState, useEffect } from 'react';
import { api, type ModelInfo } from '../lib/api';

export function ModelManager() {
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [downloadedModels, setDownloadedModels] = useState<string[]>([]);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    try {
      const [available, downloaded] = await Promise.all([
        api.getAvailableModels(),
        api.getDownloadedModels(),
      ]);
      setAvailableModels(available);
      setDownloadedModels(downloaded);
    } catch (error) {
      console.error('Failed to load models:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleDownload = async (model: ModelInfo) => {
    setDownloading(model.id);
    setProgress(0);

    try {
      await api.downloadModel(model.id, (prog) => {
        setProgress(prog.percentage);
      });
      await loadModels();
    } catch (error) {
      console.error('Download failed:', error);
      alert(`Download mislukt: ${error}`);
    } finally {
      setDownloading(null);
      setProgress(0);
    }
  };

  const handleDelete = async (modelId: string) => {
    if (!confirm('Model verwijderen?')) return;
    try {
      await api.deleteModel(modelId);
      await loadModels();
    } catch (error) {
      console.error('Delete failed:', error);
    }
  };

  const isDownloaded = (modelId: string) => {
    return downloadedModels.some(m => m.includes(modelId));
  };

  if (loading) {
    return <div className="p-4 text-zinc-400">Laden...</div>;
  }

  return (
    <div className="model-manager p-6">
      <h2 className="text-xl font-bold mb-4 text-white">AI Modellen</h2>
      <p className="text-zinc-400 text-sm mb-4">
        Download een model om lokaal te gebruiken. Alle verwerking gebeurt op jouw device.
      </p>

      <div className="space-y-3">
        {availableModels.map((model) => {
          const downloaded = isDownloaded(model.id);
          const isDownloading = downloading === model.id;

          return (
            <div key={model.id} className="bg-[#1a1a1a] border border-zinc-800 rounded-lg p-4">
              <div className="flex justify-between items-start mb-2">
                <div>
                  <h3 className="font-semibold text-white">{model.name}</h3>
                  <p className="text-sm text-zinc-400">{model.description}</p>
                  <p className="text-xs text-zinc-500 mt-1">{model.size_mb} MB</p>
                </div>
                {downloaded ? (
                  <span className="text-green-500 text-sm">✓ Gedownload</span>
                ) : (
                  <span className="text-zinc-600 text-sm">Niet gedownload</span>
                )}
              </div>

              {isDownloading && (
                <div className="mb-3">
                  <div className="bg-zinc-800 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full transition-all"
                      style={{ width: `${progress}%` }}
                    />
                  </div>
                  <p className="text-xs text-zinc-400 mt-1">{progress.toFixed(0)}%</p>
                </div>
              )}

              <div className="flex gap-2">
                {!downloaded && !isDownloading && (
                  <button
                    onClick={() => handleDownload(model)}
                    className="px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 transition-colors"
                  >
                    Download
                  </button>
                )}
                {downloaded && !isDownloading && (
                  <button
                    onClick={() => handleDelete(model.id)}
                    className="px-4 py-2 bg-red-600/20 text-red-400 border border-red-600/30 rounded text-sm hover:bg-red-600/30 transition-colors"
                  >
                    Verwijder
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      <div className="mt-6 p-4 bg-zinc-900/50 border border-zinc-800 rounded text-sm">
        <p className="font-semibold text-zinc-300 mb-1">💡 Tip:</p>
        <p className="text-zinc-400">
          Je kunt ook het script gebruiken:{' '}
          <code className="bg-zinc-800 px-2 py-1 rounded text-xs">./scripts/download-model.sh</code>
        </p>
      </div>
    </div>
  );
}
