//! Centraal register van specialisten: één waarheid voor id’s, API en routing.

use serde_json::{json, Value};

#[derive(Clone, Copy, Debug)]
pub struct SpecialistMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub description_nl: &'static str,
    pub capabilities: &'static str,
}

/// Alle routbare specialisten — nieuwe specialist = één rij hier + #[agent]-struct + dispatch in `lib.rs`.
pub const SPECIALISTS: &[SpecialistMeta] = &[
    SpecialistMeta {
        id: "code",
        name: "Code Agent",
        description_nl: "Programmeren, debuggen en software-architectuur.",
        capabilities: "read_file, list_dir, shell (read-only).",
    },
    SpecialistMeta {
        id: "schrijf",
        name: "Schrijf Agent",
        description_nl: "Nederlandse teksten: mail, rapport, samenvatting, creatief.",
        capabilities: "Geen tools.",
    },
    SpecialistMeta {
        id: "tools",
        name: "Tools Agent",
        description_nl: "Combinatie: bestanden, mappen, web, shell.",
        capabilities: "read_file, list_dir, shell, web_search.",
    },
    SpecialistMeta {
        id: "research",
        name: "Research Agent",
        description_nl: "Actuele informatie via web_search.",
        capabilities: "Alleen web_search.",
    },
    SpecialistMeta {
        id: "translate",
        name: "Vertaal Agent",
        description_nl: "Vertalen tussen Nederlands en andere talen; toon en register afstemmen.",
        capabilities: "Geen tools.",
    },
    SpecialistMeta {
        id: "analyze",
        name: "Analyse Agent",
        description_nl:
            "Bestanden en mappen begrijpen: inhoud, structuur, samenvatten wat op schijf staat.",
        capabilities: "read_file, list_dir.",
    },
    SpecialistMeta {
        id: "docs",
        name: "Documentatie Agent",
        description_nl: "README, API-docs, Markdown, technische uitleg voor ontwikkelaars.",
        capabilities: "read_file, shell (grep/find/ls).",
    },
    SpecialistMeta {
        id: "brainstorm",
        name: "Brainstorm Agent",
        description_nl: "Ideeen, concepten, creatieve invalshoeken; geen feitenclaim zonder bron.",
        capabilities: "Geen tools.",
    },
    SpecialistMeta {
        id: "data",
        name: "Data Agent",
        description_nl: "CSV/JSON/logbestanden uitlezen en interpreteren (geen SQL-engine).",
        capabilities: "read_file.",
    },
    SpecialistMeta {
        id: "general",
        name: "Algemeen",
        description_nl: "Algemene vragen zonder tools.",
        capabilities: "Geen tools.",
    },
];

/// Mesh-/ractor-experts (geen chat-routing). Alleen metadata voor `GET /api/agents` en UI.
pub const MESH_EXPERTS: &[SpecialistMeta] = &[
    SpecialistMeta {
        id: "mesh_rust",
        name: "RustExpert",
        description_nl: "Rust, async, tooling; delegeert naar mesh-peers.",
        capabilities: "Zie mesh-run (Entry → rust/frontend).",
    },
    SpecialistMeta {
        id: "mesh_frontend",
        name: "FrontendExpert",
        description_nl: "React/TS, UI; wederzijdse delegatie met RustExpert.",
        capabilities: "Zie mesh-run.",
    },
    SpecialistMeta {
        id: "mesh_research",
        name: "ResearchExpert",
        description_nl: "URLs en research-payloads (optioneel HTTP-preview).",
        capabilities: "Mesh-delegatie.",
    },
    SpecialistMeta {
        id: "mesh_schrijver",
        name: "SchrijverExpert",
        description_nl: "Schrijven op basis van research-notities en stijl.",
        capabilities: "Mesh-delegatie.",
    },
    SpecialistMeta {
        id: "mesh_pii",
        name: "PIIStripperExpert",
        description_nl: "PII-maskering / privacy-filter sidecar.",
        capabilities: "Mesh-delegatie.",
    },
    SpecialistMeta {
        id: "mesh_reviewer",
        name: "ReviewerExpert",
        description_nl: "Preflight: regels, TODO’s, criteria.",
        capabilities: "Mesh-delegatie.",
    },
    SpecialistMeta {
        id: "mesh_document",
        name: "DocumentOrchestrator",
        description_nl: "Meerstaps documentflow (research → review).",
        capabilities: "Mesh-delegatie.",
    },
];

/// Vaste id-lijst (sync met SPECIALISTS).
pub const ROUTING_IDS: &[&str] = &[
    "code",
    "schrijf",
    "tools",
    "research",
    "translate",
    "analyze",
    "docs",
    "brainstorm",
    "data",
    "general",
];

/// Expliciete keuze in de UI / API (geen orchestrator).
pub fn is_registered_id(raw: &str) -> bool {
    let key = raw.trim().to_lowercase();
    ROUTING_IDS.iter().any(|&id| id == key.as_str())
}

/// Standaard Ollama-model als het request géén `model` meestuurt (`OLLAMA_MODEL` of fallback).
pub fn default_ollama_model() -> String {
    std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen3-coder:latest".to_string())
}

pub fn ollama_base_url() -> String {
    std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string())
}

pub fn orchestrator_candidate_line() -> String {
    let mut s = String::from("Toegestane waarde voor \"agent\" (exact, kleine letters):\n");
    for m in SPECIALISTS {
        s.push_str(&format!(
            "- \"{}\": {} — {}\n",
            m.id, m.description_nl, m.capabilities
        ));
    }
    s
}

/// Normaliseer ruwe LLM-output naar een canonical ROUTING_IDS-waarde.
pub fn normalize_route(raw: &str) -> &'static str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "general";
    }

    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        if let Some(a) = v.get("agent").and_then(|x| x.as_str()) {
            return normalize_route(a);
        }
    }

    let t = trimmed.to_lowercase();

    match t.as_str() {
        "code" | "code_expert" => "code",
        "schrijf" | "schrijf_expert" => "schrijf",
        "tools" | "tools_expert" => "tools",
        "research" | "research_expert" => "research",
        "translate" | "translate_expert" | "vertaal" => "translate",
        "analyze" | "analyze_expert" | "analyse" => "analyze",
        "docs" | "docs_expert" | "documentation" => "docs",
        "brainstorm" | "brainstorm_expert" => "brainstorm",
        "data" | "data_expert" => "data",
        "general" | "general_assistant" => "general",
        _ => fuzzy_fallback(&t),
    }
}

fn fuzzy_fallback(t: &str) -> &'static str {
    if t.contains("vertaal") || t.contains("translate") || t.contains("engels") {
        return "translate";
    }
    if t.contains("brainstorm") || t.contains("idee") || t.contains("concept") {
        return "brainstorm";
    }
    if t.contains("readme") || t.contains("documentatie") || t.contains("api doc") {
        return "docs";
    }
    if t.contains("csv") || t.contains("json") || t.contains("dataset") || t.contains("tabular") {
        return "data";
    }
    if t.contains("analyse") || t.contains("analyze") || t.contains("bestand inhoud") {
        return "analyze";
    }
    if t.contains("research") || t.contains("nieuws") || t.contains("actueel") {
        return "research";
    }
    if t.contains("schrijf") || t.contains("tekst") || t.contains("e-mail") {
        return "schrijf";
    }
    if t.contains("tools") || t.contains("bestand") || t.contains("zoek web") {
        return "tools";
    }
    if t.contains("code") || t.contains("program") || t.contains("debug") {
        return "code";
    }
    "general"
}

pub fn agents_http_json() -> Vec<Value> {
    let mut out = vec![json!({
        "id": "orchestrator",
        "name": "Orchestrator",
        "description": "Analyseert de vraag en kiest exact één specialist-ID (JSON output).",
        "kind": "orchestrator",
        "selectable": true,
    })];
    for m in SPECIALISTS {
        out.push(json!({
            "id": m.id,
            "name": m.name,
            "description": m.description_nl,
            "kind": "chat",
            "selectable": true,
        }));
    }
    for m in MESH_EXPERTS {
        out.push(json!({
            "id": m.id,
            "name": m.name,
            "description": format!("{} {}", m.description_nl, m.capabilities),
            "kind": "mesh",
            "selectable": false,
        }));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_len_matches() {
        assert_eq!(SPECIALISTS.len(), ROUTING_IDS.len());
        for (m, &id) in SPECIALISTS.iter().zip(ROUTING_IDS.iter()) {
            assert_eq!(m.id, id, "VOLGORDE mismatch registry");
        }
    }

    #[test]
    fn registered_id_helper_works() {
        assert!(super::is_registered_id("translate"));
        assert!(!super::is_registered_id("unknown"));
    }

    #[test]
    fn normalize_json_nested() {
        assert_eq!(normalize_route(r#"{"agent":"research"}"#), "research");
    }

    #[test]
    fn normalize_aliases() {
        assert_eq!(normalize_route("translate_expert"), "translate");
        assert_eq!(normalize_route("VERTAAL"), "translate");
    }

    #[test]
    fn agents_json_includes_mesh() {
        let v = agents_http_json();
        let mesh: Vec<_> = v
            .iter()
            .filter(|x| x.get("kind").and_then(|k| k.as_str()) == Some("mesh"))
            .collect();
        assert_eq!(mesh.len(), MESH_EXPERTS.len());
        assert!(v
            .iter()
            .any(|x| x["id"] == json!("mesh_rust") && x["selectable"] == json!(false)));
    }
}
