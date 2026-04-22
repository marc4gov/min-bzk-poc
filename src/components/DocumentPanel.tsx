import React, { useState, useEffect } from 'react';
import { Button } from './ui/Button';
import { open } from '@tauri-apps/plugin-dialog';
import type { Document } from '../types';
import { api } from '../lib/api';

export const DocumentPanel: React.FC = () => {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [selectedDoc, setSelectedDoc] = useState<Document | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);

  useEffect(() => {
    loadDocuments();
  }, []);

  const loadDocuments = async () => {
    try {
      const docs = await api.getDocuments();
      setDocuments(docs);
    } catch (error) {
      console.error('Failed to load documents:', error);
    }
  };

  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Documents',
            extensions: ['txt', 'md'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        await loadDocument(selected);
      }
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  const loadDocument = async (filePath: string) => {
    try {
      await api.extractText(filePath);
      await loadDocuments();
      const docs = await api.getDocuments();
      setDocuments(docs);
      setSelectedDoc(docs.find((d) => d.file_path === filePath) || null);
    } catch (error) {
      console.error('Failed to load document:', error);
    }
  };

  const handleAnalyze = async () => {
    if (!selectedDoc) return;
    setIsAnalyzing(true);
    setTimeout(() => setIsAnalyzing(false), 1000);
  };

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a]">
      <div className="p-4 border-b border-zinc-800">
        <h2 className="text-lg font-semibold text-white mb-2">Documenten</h2>
        <Button onClick={handleFileSelect}>+ Document toevoegen</Button>
      </div>

      <div className="flex-1 flex overflow-hidden">
        <div className="w-64 border-r border-zinc-800 overflow-y-auto p-2">
          {documents.map((doc) => (
            <div
              key={doc.id}
              onClick={() => setSelectedDoc(doc)}
              className={`p-3 rounded-lg cursor-pointer transition-colors ${
                selectedDoc?.id === doc.id
                  ? 'bg-indigo-600 text-white'
                  : 'hover:bg-zinc-800 text-zinc-300'
              }`}
            >
              <div className="truncate text-sm">{doc.filename}</div>
              <div className="text-xs opacity-60 mt-1">
                {new Date(doc.created_at).toLocaleDateString('nl-NL')}
              </div>
            </div>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          {selectedDoc ? (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-xl font-semibold">{selectedDoc.filename}</h3>
                <Button onClick={handleAnalyze} disabled={isAnalyzing}>
                  {isAnalyzing ? 'Analyseert...' : 'Analyseer'}
                </Button>
              </div>
              <div className="bg-zinc-900 rounded-lg p-4">
                <pre className="text-sm text-zinc-300 whitespace-pre-wrap">
                  {selectedDoc.extracted_text || selectedDoc.content_preview}
                </pre>
              </div>
            </div>
          ) : (
            <div className="flex items-center justify-center h-full text-zinc-500">
              <p>Selecteer een document om te bekijken</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
