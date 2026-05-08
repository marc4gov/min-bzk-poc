import { useState } from 'react';
import { Button } from './ui/Button';
import { Textarea } from './ui/Textarea';
import { agentApi } from '../lib/agent-api';
import { useAgentDiagnostics } from '../AgentDiagnosticsProvider';

/** Twee experts alleen — snelle demo (`full_mesh=false`). */
const PRESETS_MINIMAL: { label: string; text: string; hint?: string }[] = [
  {
    label: 'Rust + React + WASM',
    text: 'How do I use Rust WebAssembly together with React hooks?',
    hint: 'Triage naar RustExpert; delegatie FrontendExpert mogelijk',
  },
  {
    label: 'Alleen Rust',
    text: 'Explain Rust ownership and borrowing in one short paragraph.',
    hint: 'RustExpert',
  },
  {
    label: 'Alleen frontend',
    text: 'What is a React hook for component-local state?',
    hint: 'FrontendExpert',
  },
];

/**
 * `full_mesh=true`: Entry registreert alle mesh-experts; triage gebruikt keywords uit `agent-service`/entry.rs.
 */
const PRESETS_FULL: { label: string; text: string; hint?: string }[] = [
  ...PRESETS_MINIMAL,
  {
    label: 'Research (URLs/scrape)',
    text: 'Research and gather summaries from https://example.org about climate policies',
    hint: 'Triage → research',
  },
  {
    label: 'Schrijven (draft)',
    text: 'Write a short Dutch paragraph about tides for a blog draft',
    hint: 'Triage → write',
  },
  {
    label: 'PII scrub',
    text: 'Anonymize and scrub email test@firma.nl and phone numbers from this text',
    hint: 'Triage → pii',
  },
  {
    label: 'Review / edit',
    text: 'Critique and edit this one-liner for clarity',
    hint: 'Triage → review',
  },
  {
    label: 'SQL / database',
    text: 'Best practices for SQLite schema design and migrations',
    hint: 'database → gekoppeld aan ResearchExpert voor deze demo',
  },
  {
    label: 'NL: vraag + URL → documentketen',
    text: 'Kun je op basis van https://www.rust-lang.org een korte samenvatting voor het team schrijven?',
    hint: 'Inferentie naar CreateDocument‑pipeline (geen keyword „document”)',
  },
  {
    label: 'Document (via triage)',
    text: 'Create document from https://example.org summarizing the homepage for stakeholders',
    hint: 'Triage document → CreateDocument-pipeline (URL in de vraag)',
  },
  {
    label: 'Algemeen (default)',
    text: 'What is good morning in Japanese?',
    hint: 'general → FrontendExpert stub',
  },
];

const ROUTE_TWO_EXPERTS = [
  {
    name: 'RustExpert',
    capabilities: ['rust', 'memory', 'thread', 'async', 'tokio'],
    note: 'Klein mesh-pad alleen Rust + Frontend beschikbaar',
  },
  {
    name: 'FrontendExpert',
    capabilities: ['frontend', 'ui', 'react', 'vue'],
    note: '',
  },
] as const;

const ROUTE_FULL = [
  { name: 'RustExpert — rust', caps: '(Entry-key: rust)' },
  { name: 'FrontendExpert — frontend', caps: '(ook general)' },
  { name: 'ResearchExpert — research', caps: '+ database-triage' },
  { name: 'SchrijverExpert — write', caps: '' },
  { name: 'PIIStripperExpert — pii', caps: '' },
  { name: 'ReviewerExpert — review', caps: '' },
  { name: 'DocumentOrchestrator', caps: 'triage „document” of apart document-endpoint' },
] as const;

const DEFAULT_DOC_URLS = `https://example.org
https://www.rust-lang.org`;

export function MeshDemoPanel() {
  const { agentServiceAvailable, recheckAgentService } = useAgentDiagnostics();
  const [query, setQuery] = useState(PRESETS_MINIMAL[0].text);
  const [fullMesh, setFullMesh] = useState(true);
  const [timeoutSecs, setTimeoutSecs] = useState(90);
  const [loading, setLoading] = useState(false);
  const [lastOk, setLastOk] = useState<boolean | null>(null);
  const [lastText, setLastText] = useState('');
  const [documentUrlsRaw, setDocumentUrlsRaw] = useState(DEFAULT_DOC_URLS);
  const [docTimeoutSecs, setDocTimeoutSecs] = useState(120);

  const presets = fullMesh ? PRESETS_FULL : PRESETS_MINIMAL;

  const runMesh = async () => {
    const q = query.trim();
    if (!q || loading) return;
    setLoading(true);
    setLastOk(null);
    setLastText('');
    try {
      const out = await agentApi.runMeshDemo(q, timeoutSecs, fullMesh);
      setLastOk(out.ok);
      if (out.ok && out.result != null) {
        setLastText(out.result);
      } else if (out.error != null) {
        setLastText(out.error);
      } else {
        setLastText('Geen inhoud.');
      }
    } catch (e) {
      setLastOk(false);
      setLastText(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const runDocumentPipeline = async () => {
    const urls = documentUrlsRaw
      .split(/\r?\n/)
      .map((s) => s.trim())
      .filter(Boolean);
    if (urls.length === 0 || loading) return;
    setLoading(true);
    setLastOk(null);
    setLastText('');
    try {
      const out = await agentApi.runMeshDocument(urls, docTimeoutSecs);
      setLastOk(out.ok);
      if (out.ok && out.result != null) {
        setLastText(out.result);
      } else if (out.error != null) {
        setLastText(out.error);
      } else {
        setLastText('Geen inhoud.');
      }
    } catch (e) {
      setLastOk(false);
      setLastText(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="mesh-demo p-6 max-w-3xl mx-auto pb-96">
      <div className="flex flex-wrap items-center gap-3 mb-4">
        <h2 className="text-xl font-bold text-white">Mesh-demo (actor-mesh)</h2>
        <Button type="button" variant="ghost" className="text-xs" onClick={() => void recheckAgentService()}>
          Verbinding controleren
        </Button>
      </div>

      <label className="mb-4 flex cursor-pointer items-start gap-3 rounded-lg border border-indigo-900/80 bg-indigo-950/30 px-3 py-2 text-sm text-zinc-200">
        <input
          type="checkbox"
          className="mt-1 accent-indigo-500"
          checked={fullMesh}
          onChange={(e) => {
            const v = e.target.checked;
            setFullMesh(v);
            setQuery((v ? PRESETS_FULL : PRESETS_MINIMAL)[0]?.text ?? '');
          }}
          disabled={loading}
        />
        <span>
          <strong>Volledige mesh-topologie</strong> — registreert Research, Schrijver, PII-stripper en Reviewer bij
          Entry (+ Rust/Frontend-peergrafiek). Uit geschakeld: alleen de klassieke tweeledige demo zoals eerder (
          <code className="text-indigo-300/90">full_mesh=false</code>).
        </span>
      </label>

      <details className="mb-4 rounded-lg border border-zinc-800 bg-zinc-950/60 px-3 py-2 text-sm text-zinc-300 open:pb-3">
        <summary className="cursor-pointer select-none font-medium text-zinc-200">
          Welke experts in <code className="text-zinc-400">POST /api/mesh/demo</code>
        </summary>
        {!fullMesh ? (
          <>
            <p className="mt-2 text-zinc-400">
              Twee experts met peer-koppeling Rust ↔ Frontend (<code className="text-zinc-500">run_mesh_once</code>).
            </p>
            <ul className="mt-2 list-inside list-disc space-y-2 text-zinc-400">
              {ROUTE_TWO_EXPERTS.map((ex) => (
                <li key={ex.name}>
                  <span className="text-zinc-200">{ex.name}</span>{' '}
                  <code className="text-xs text-emerald-500/90">{ex.capabilities.join(', ')}</code> {ex.note}
                </li>
              ))}
            </ul>
          </>
        ) : (
          <ul className="mt-2 list-inside list-disc space-y-1 text-xs text-zinc-400">
            {ROUTE_FULL.map((r) => (
              <li key={r.name}>
                <span className="font-medium text-zinc-300">{r.name}</span> {r.caps}
              </li>
            ))}
          </ul>
        )}
        <p className="mt-2 text-xs text-zinc-500">
          Chat-route (`GET /api/agents`) is los van deze mesh. DocumentOrchestrator draait via{' '}
          <code className="text-zinc-500">POST /api/mesh/demo/document</code> hieronder (CreateDocument‑keten).
        </p>
      </details>

      <div className="mb-4 rounded-lg border border-zinc-700/70 bg-zinc-900/40 px-3 py-2 text-xs leading-relaxed text-zinc-400">
        <strong className="text-zinc-300">Entry‑triage</strong> (
        <code className="text-zinc-500">mesh/entry.rs</code>): eerste match — o.a.&nbsp;
        rust → frontend → database → research → write → pii → review → document → <strong className="text-zinc-200">general</strong>{' '}
        (met volledige mesh actoren gekoppeld).
      </div>

      <p className="mb-4 text-sm text-zinc-400">
        Ollama: <code className="text-zinc-300">OLLAMA_*</code> / <code className="text-zinc-300">MESH_USE_OLLAMA</code>.
        PII sidecar voor scrub: <code className="text-zinc-300">PII_PRIVACY_FILTER_URL</code>. Logs: paneel rechtsonder.
      </p>

      {!agentServiceAvailable && (
        <div className="mb-4 rounded-lg border border-amber-800/80 bg-amber-950/40 px-3 py-2 text-sm text-amber-200">
          Agent-service niet bereikbaar — start <code className="text-amber-100">agent-service</code>.
        </div>
      )}
      {agentServiceAvailable && (
        <div className="mb-3 rounded-lg border border-emerald-900/80 bg-emerald-950/30 px-3 py-2 text-sm text-emerald-200/90">
          Agent-service bereikbaar.
        </div>
      )}

      <p className="mb-2 text-xs font-medium uppercase tracking-wide text-zinc-500">
        Scenario’s (plaats tekst + „Mesh uitvoeren”)
      </p>
      <div className="mb-3 flex flex-wrap gap-2">
        {presets.map((p) => (
          <Button
            key={p.label}
            type="button"
            variant="ghost"
            className="text-xs"
            title={p.hint ?? p.label}
            onClick={() => setQuery(p.text)}
          >
            {p.label}
          </Button>
        ))}
      </div>

      <Textarea
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        className="mb-3 min-h-[100px] font-mono text-sm"
        placeholder="Query voor SubmitRequest naar Entry…"
        disabled={loading}
      />

      <div className="mb-6 flex flex-wrap items-center gap-4">
        <label className="flex items-center gap-2 text-sm text-zinc-400">
          Timeout mesh (s)
          <input
            type="number"
            min={5}
            max={600}
            value={timeoutSecs}
            onChange={(e) => setTimeoutSecs(Number(e.target.value) || 30)}
            className="w-20 rounded border border-zinc-700 bg-zinc-900 px-2 py-1 text-sm text-white"
            disabled={loading}
          />
        </label>
        <Button type="button" onClick={() => void runMesh()} disabled={loading || !query.trim()}>
          {loading ? 'Bezig…' : 'Mesh uitvoeren'}
        </Button>
      </div>

      <div className="mb-6 border-t border-zinc-800 pt-6">
        <h3 className="mb-1 text-sm font-semibold text-zinc-200">Volledige document-pipeline</h3>
        <p className="mb-3 text-xs text-zinc-500">
          Research → Schrijven → PII-strip → Review over <code className="text-zinc-400">POST /api/mesh/demo/document</code>.
        </p>
        <Textarea
          value={documentUrlsRaw}
          onChange={(e) => setDocumentUrlsRaw(e.target.value)}
          className="mb-3 min-h-[80px] font-mono text-xs"
          placeholder={'Eén URL per regel'}
          disabled={loading}
        />
        <div className="flex flex-wrap items-center gap-4">
          <label className="flex items-center gap-2 text-sm text-zinc-400">
            Timeout (s)
            <input
              type="number"
              min={15}
              max={600}
              value={docTimeoutSecs}
              onChange={(e) => setDocTimeoutSecs(Number(e.target.value) || 120)}
              className="w-24 rounded border border-zinc-700 bg-zinc-900 px-2 py-1 text-sm text-white"
              disabled={loading}
            />
          </label>
          <Button type="button" variant="secondary" onClick={() => void runDocumentPipeline()} disabled={loading}>
            {loading ? 'Bezig…' : 'Document-pipeline uitvoeren'}
          </Button>
        </div>
      </div>

      {lastOk !== null && (
        <div
          className={`rounded-lg border px-4 py-3 text-sm whitespace-pre-wrap ${
            lastOk
              ? 'border-emerald-800 bg-emerald-950/30 text-emerald-100'
              : 'border-red-900/80 bg-red-950/30 text-red-100'
          }`}
        >
          {lastText}
        </div>
      )}
    </div>
  );
}
