import React, { useState } from 'react';
import { Button } from './ui/Button';
import { Textarea } from './ui/Textarea';
import { open } from '@tauri-apps/plugin-dialog';

interface DocumentUploadProps {
  onUploadStart: (filename: string, content: string, instructions: string) => void;
  disabled?: boolean;
}

export const DocumentUpload: React.FC<DocumentUploadProps> = ({ onUploadStart, disabled }) => {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [instructions, setInstructions] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [dragActive, setDragActive] = useState(false);

  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Documenten',
            extensions: ['txt', 'pdf', 'docx', 'md'],
          },
        ],
      });
      if (selected && typeof selected === 'string') {
        setSelectedFile(selected);
      }
    } catch (error) {
      console.error('Bestandsselectie mislukt:', error);
    }
  };

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true);
    } else if (e.type === 'dragleave') {
      setDragActive(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);

    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      // For web; Tauri file handling is different
      // This would need Tauri-specific path handling
      console.log('File dropped:', e.dataTransfer.files[0].name);
    }
  };

  const handleUpload = async () => {
    if (!selectedFile) return;

    setIsProcessing(true);
    try {
      // Import api dynamically to avoid circular deps
      const { api } = await import('../lib/api');
      const result = await api.uploadDocumentForImprovement(
        selectedFile,
        instructions || 'Verbeter dit document voor duidelijkheid en professionaliteit.'
      );

      // Use the instructions we sent, not from result (API doesn't echo it back)
      const instructionsUsed = instructions || 'Verbeter dit document voor duidelijkheid en professionaliteit.';
      onUploadStart(result.filename, result.content, instructionsUsed);
      setSelectedFile(null);
      setInstructions('');
    } catch (error) {
      console.error('Upload mislukt:', error);
      alert('Upload mislukt: ' + (error instanceof Error ? error.message : 'Onbekende fout'));
    } finally {
      setIsProcessing(false);
    }
  };

  const filename = selectedFile?.split('/').pop()?.split('\\').pop() || '';

  return (
    <div className="border border-zinc-700 rounded-lg p-4 bg-[#1a1a1a]">
      <h3 className="text-sm font-medium text-zinc-300 mb-3">Document verbeteren</h3>

      {/* File selection */}
      <div
        className={`border-2 border-dashed rounded-lg p-4 text-center transition-colors ${
          dragActive
            ? 'border-indigo-500 bg-indigo-500/10'
            : 'border-zinc-700 hover:border-zinc-600'
        }`}
        onDragEnter={handleDrag}
        onDragLeave={handleDrag}
        onDragOver={handleDrag}
        onDrop={handleDrop}
      >
        {selectedFile ? (
          <div className="text-sm text-zinc-300">
            <svg className="w-8 h-8 mx-auto mb-2 text-indigo-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p className="font-medium">{filename}</p>
            <button
              type="button"
              onClick={() => setSelectedFile(null)}
              className="text-xs text-zinc-500 hover:text-zinc-400 mt-1 underline"
            >
              Verwijderen
            </button>
          </div>
        ) : (
          <div>
            <svg className="w-8 h-8 mx-auto mb-2 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            <p className="text-sm text-zinc-400 mb-2">Sleep een bestand of klik om te selecteren</p>
            <Button
              type="button"
              onClick={handleFileSelect}
              disabled={disabled}
              variant="secondary"
              className="text-xs"
            >
              Bladeren
            </Button>
          </div>
        )}
      </div>

      {/* Instructions */}
      <div className="mt-3">
        <label className="block text-xs text-zinc-400 mb-1">Instructies (optioneel)</label>
        <Textarea
          value={instructions}
          onChange={(e) => setInstructions(e.target.value)}
          placeholder="Bijv: 'Maak de tekst formeler', 'Verbeter de structuur', 'Vereenvoudig de taal'"
          className="min-h-[60px] text-sm"
          disabled={!selectedFile || disabled}
        />
      </div>

      {/* Upload button */}
      <Button
        type="button"
        onClick={handleUpload}
        disabled={!selectedFile || isProcessing || disabled}
        className="w-full mt-3"
      >
        {isProcessing ? (
          <>
            <svg className="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Verwerken...
          </>
        ) : (
          <>
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            Verbeter document
          </>
        )}
      </Button>

      <p className="text-xs text-zinc-500 mt-2">
        Ondersteunde formaten: TXT, PDF, DOCX, MD. Max ~10MB.
      </p>
    </div>
  );
};
