use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub agent_type: String,
    pub prompt: String,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    pub tasks: Vec<AgentTask>,
    pub reasoning: String,
}

pub struct OrchestratorAgent {
    llm: std::sync::Arc<dyn crate::llm::LLMProvider>,
}

impl OrchestratorAgent {
    pub fn new(llm: std::sync::Arc<dyn crate::llm::LLMProvider>) -> Self {
        Self { llm }
    }

    pub async fn analyze(&self, query: &str) -> Result<OrchestrationPlan> {
        let system_prompt = r#"
Je bent een orchestrator voor een multi-agent AI systeem. Je hebt deze agents beschikbaar:

1. **code**: Voor code generatie, debugging, technische uitleg
2. **schrijf**: Voor teksten schrijven, redigeren, samenvatten
3. **tools**: Voor bestanden lezen, web search, shell commands

Analyseer de gebruikersvraag en bepaal welke agents nodig zijn.
Geef je antwoord in JSON formaat:
{
  "reasoning": "korte uitleg waarom deze agents",
  "agents": ["agent1", "agent2"]
}

Alleen deze agent types zijn geldig: code, schrijf, tools.
"#;

        let prompt = format!("{}\n\nGebruikersvraag: {}", system_prompt, query);

        let response = self.llm.generate(crate::llm::LLMRequest {
            prompt,
            max_tokens: Some(300),
            temperature: Some(0.3),
        }).await?;

        let agents = self.parse_agent_selection(&response.content, query);

        let tasks = agents.iter().map(|agent_type| AgentTask {
            agent_type: agent_type.clone(),
            prompt: query.to_string(),
            priority: 0,
        }).collect();

        Ok(OrchestrationPlan {
            tasks,
            reasoning: response.content,
        })
    }

    fn parse_agent_selection(&self, llm_response: &str, query: &str) -> Vec<String> {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(llm_response) {
            if let Some(agents) = json["agents"].as_array() {
                let valid_agents: Vec<String> = agents.iter()
                    .filter_map(|a| a.as_str())
                    .filter(|s| ["code", "schrijf", "tools"].contains(s))
                    .map(String::from)
                    .collect();

                if !valid_agents.is_empty() {
                    return valid_agents;
                }
            }
        }

        // Fallback: keyword-based classification
        let mut selected = Vec::new();
        let query_lower = query.to_lowercase();

        let code_keywords = ["code", "functie", "function", "bug", "debug", "python", "rust", "javascript", "programmeer"];
        if code_keywords.iter().any(|kw| query_lower.contains(kw)) {
            selected.push("code".to_string());
        }

        let write_keywords = ["schrijf", "tekst", "mail", "brief", "essay", "verhaal", "samenvatten"];
        if write_keywords.iter().any(|kw| query_lower.contains(kw)) {
            selected.push("schrijf".to_string());
        }

        let tools_keywords = ["bestand", "lees", "zoek", "search", "file", "web", "directory"];
        if tools_keywords.iter().any(|kw| query_lower.contains(kw)) {
            selected.push("tools".to_string());
        }

        if selected.is_empty() {
            selected.push("schrijf".to_string());
        }

        selected
    }
}
