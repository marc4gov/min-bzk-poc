import { useState } from 'react';
import { ChatView } from './components/ChatView';
import { HistorySidebar } from './components/HistorySidebar';
import { DocumentPanel } from './components/DocumentPanel';
import { Button } from './components/ui/Button';

type Tab = 'chat' | 'documents';

function App() {
  const [currentHistoryId, setCurrentHistoryId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>('chat');

  const handleNewChat = () => {
    setCurrentHistoryId(null);
  };

  return (
    <div className="flex h-screen bg-[#0a0a0a]">
      <HistorySidebar
        currentHistoryId={currentHistoryId}
        onSelectHistory={setCurrentHistoryId}
        onNewChat={handleNewChat}
      />

      <div className="flex-1 flex flex-col">
        <header className="h-14 bg-[#1a1a1a] border-b border-zinc-800 flex items-center px-4">
          <div className="flex space-x-2">
            <Button
              variant={activeTab === 'chat' ? 'primary' : 'ghost'}
              onClick={() => setActiveTab('chat')}
            >
              Chat
            </Button>
            <Button
              variant={activeTab === 'documents' ? 'primary' : 'ghost'}
              onClick={() => setActiveTab('documents')}
            >
              Documenten
            </Button>
          </div>
          <div className="ml-auto">
            <span className="text-sm text-zinc-400">Local Assistant</span>
          </div>
        </header>

        <main className="flex-1 overflow-hidden">
          {activeTab === 'chat' ? (
            <ChatView
              historyId={currentHistoryId}
              onHistoryChange={setCurrentHistoryId}
            />
          ) : (
            <DocumentPanel />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
