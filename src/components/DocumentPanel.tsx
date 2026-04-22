import React, { useState, useEffect } from 'react';
import { Button } from './ui/Button';
import { open } from '@tauri-apps/plugin-dialog';
import type { Document } from '../types';
import { api } from '../lib/api';

export const DocumentPanel: React.FC = () => {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [selectedDoc, setSelectedDoc] = useState<Document | null>(null);
  const [isLoading, setIsLoading] = useState(false);

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
        multiple: true,
        filters: [
          {
            name: 'Text & Documents',
            extensions: ['txt', 'md', 'json', 'csv', 'log'],
          },
        ],
      });

      if (Array.isArray(selected)) {
        for (const filePath of selected) {
          await loadDocument(filePath);
        }
      } else if (selected && typeof selected === 'string') {
        await loadDocument(selected);
      }
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  const loadDocument = async (filePath: string) => {
    setIsLoading(true);
    try {
      await api.extractText(filePath);
      await loadDocuments();
      const docs = await api.getDocuments();
      setDocuments(docs);
      const newlyAdded = docs.find((d) => d.file_path === filePath);
      if (newlyAdded) {
        setSelectedDoc(newlyAdded);
      }
    } catch (error) {
      console.error('Failed to load document:', error);
      alert('Kon bestand niet lezen. Controleer of het een geldig tekstbestand is.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Document verwijderen?')) return;
    // TODO: Add delete API call
    setDocuments(documents.filter((d) => d.id !== id));
    if (selectedDoc?.id === id) {
      setSelectedDoc(null);
    }
  };

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a]">
      <div className="p-4 border-b border-zinc-800">
        <h2 className="text-lg font-semibold text-white mb-2">Documenten</h2>
        <Button onClick={handleFileSelect} disabled={isLoading}>
          {isLoading ? 'Laden...' : '+ Document toevoegen'}
        </Button>
        <p className="text-xs text-zinc-500 mt-2">
          Ondersteunde bestanden: .txt, .md, .json, .csv, .log
        </p>
      </div>

      <div className="flex-1 flex overflow-hidden">
        <div className="w-64 border-r border-zinc-800 overflow-y-auto p-2">
          {documents.length === 0 ? (
            <div className="text-center py-8 text-zinc-500">
              <svg className="w-12 h-12 mx-auto mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p className="text-sm">Nog geen documenten</p>
              <p className="text-xs mt-1">Klik op de knop om bestanden toe te voegen</p>
            </div>
          ) : (
            <div className="space-y-1">
              {documents.map((doc) => (
                <div
                  key={doc.id}
                  onClick={() => setSelectedDoc(doc)}
                  className="group p-3 rounded-lg cursor-pointer transition-colors hover:bg-zinc-800 text-zinc-300"
                  style={{
                    backgroundColor: selectedDoc?.id === doc.id ? 'rgb(55, 65, 81)' : undefined,
                    color: selectedDoc?.id === doc.id ? 'white' : undefined,
                  }}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex-1 truncate text-sm">{doc.filename}</div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(doc.id);
                      }}
                      className="opacity-0 group-hover:opacity-100 text-zinc-500 hover:text-red-400 transition-opacity"
                    >
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                  <div className="text-xs opacity-60 mt-1">
                    {new Date(doc.created_at).toLocaleDateString('nl-NL', {
                      day: '2-digit',
                      month: 'short',
                    })}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          {selectedDoc ? (
            <div className="space-y-4">
              <div>
                <h3 className="text-xl font-semibold">{selectedDoc.filename}</h3>
                <p className="text-xs text-zinc-400 mt-1">
                  {(selectedDoc.extracted_text || selectedDoc.content_preview).length} tekens
                </p>
              </div>
              <div className="bg-zinc-900 rounded-lg p-4 border border-zinc-800">
                <pre className="text-sm text-zinc-300 whitespace-pre-wrap break-words font-mono">
                  {selectedDoc.extracted_text || selectedDoc.content_preview}
                </pre>
              </div>
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center h-full text-zinc-500">
              <svg className="w-16 h-16 mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p>Selecteer een document om te bekijken</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
