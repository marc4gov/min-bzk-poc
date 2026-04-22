import React, { useState, useEffect } from 'react';
import { Button } from './ui/Button';
import { Input } from './ui/Input';
import { Textarea } from './ui/Textarea';
import type { Template } from '../types';

interface TemplateManagerProps {
  onApplyTemplate: (template: Template) => void;
}

export const TemplateManager: React.FC<TemplateManagerProps> = ({ onApplyTemplate }) => {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [newTemplateName, setNewTemplateName] = useState('');
  const [newTemplateDesc, setNewTemplateDesc] = useState('');
  const [newTemplatePrompt, setNewTemplatePrompt] = useState('');

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    try {
      // For now, use builtin templates since storage.get_templates needs storage access
      const builtin: Template[] = [
        {
          id: '1',
          name: 'Email opstellen',
          description: 'Schrijf een professionele email',
          prompt_template: 'Schrijf een professionele email over het volgende onderwerp: {{topic}}\\n\\nContext: {{context}}',
          is_builtin: true,
        },
        {
          id: '2',
          name: 'Samenvatting',
          description: 'Maak een samenvatting van de volgende tekst',
          prompt_template: 'Maak een korte samenvatting van de volgende tekst:\\n\\n{{text}}',
          is_builtin: true,
        },
        {
          id: '3',
          name: 'Code uitleg',
          description: 'Leg code uit in eenvoudige taal',
          prompt_template: 'Leg de volgende code uit in eenvoudige taal:\\n\\n```\\n{{code}}\\n```',
          is_builtin: true,
        },
      ];
      setTemplates(builtin);
    } catch (error) {
      console.error('Failed to load templates:', error);
    }
  };

  const handleCreateTemplate = () => {
    if (!newTemplateName.trim()) return;
    const newTemplate: Template = {
      id: Date.now().toString(),
      name: newTemplateName,
      description: newTemplateDesc,
      prompt_template: newTemplatePrompt,
      is_builtin: false,
    };
    setTemplates([...templates, newTemplate]);
    setNewTemplateName('');
    setNewTemplateDesc('');
    setNewTemplatePrompt('');
    setIsCreating(false);
  };

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a]">
      <div className="p-4 border-b border-zinc-800 flex items-center justify-between">
        <h2 className="text-lg font-semibold text-white">Templates</h2>
        <Button size="sm" onClick={() => setIsCreating(!isCreating)}>
          {isCreating ? 'Annuleren' : '+ Nieuw'}
        </Button>
      </div>

      <div className="flex-1 flex overflow-hidden">
        <div className="w-64 border-r border-zinc-800 overflow-y-auto p-2">
          {templates.map((template) => (
            <div
              key={template.id}
              onClick={() => setSelectedTemplate(template)}
              className={`p-3 rounded-lg cursor-pointer transition-colors ${
                selectedTemplate?.id === template.id
                  ? 'bg-indigo-600 text-white'
                  : 'hover:bg-zinc-800 text-zinc-300'
              }`}
            >
              <div className="text-sm font-medium">{template.name}</div>
              <div className="text-xs opacity-60 mt-1">{template.description}</div>
            </div>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          {isCreating ? (
            <div className="space-y-4">
              <h3 className="text-xl font-semibold">Nieuw Template</h3>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Naam</label>
                <Input
                  value={newTemplateName}
                  onChange={(e) => setNewTemplateName(e.target.value)}
                  placeholder="Template naam"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Beschrijving</label>
                <Input
                  value={newTemplateDesc}
                  onChange={(e) => setNewTemplateDesc(e.target.value)}
                  placeholder="Korte beschrijving"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Prompt Template</label>
                <Textarea
                  value={newTemplatePrompt}
                  onChange={(e) => setNewTemplatePrompt(e.target.value)}
                  placeholder="{'{{variabele}}'} voor placeholders"
                  className="min-h-[150px]"
                />
                <p className="text-xs text-zinc-500 mt-1">
                  Gebruik <code className="bg-zinc-800 px-1 rounded">{'{{variabele}}'}</code> voor placeholders
                </p>
              </div>
              <div className="flex space-x-2">
                <Button onClick={handleCreateTemplate}>Opslaan</Button>
                <Button variant="ghost" onClick={() => setIsCreating(false)}>Annuleren</Button>
              </div>
            </div>
          ) : selectedTemplate ? (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-xl font-semibold">{selectedTemplate.name}</h3>
                  <p className="text-sm text-zinc-400">{selectedTemplate.description}</p>
                </div>
                <Button onClick={() => onApplyTemplate(selectedTemplate)}>
                  Toepassen
                </Button>
              </div>
              <div className="bg-zinc-900 rounded-lg p-4">
                <h4 className="text-sm font-medium text-zinc-400 mb-2">Prompt Template:</h4>
                <pre className="text-sm text-white whitespace-pre-wrap font-mono">
                  {selectedTemplate.prompt_template}
                </pre>
              </div>
            </div>
          ) : (
            <div className="flex items-center justify-center h-full text-zinc-500">
              <p>Selecteer een template om te bekijken</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
