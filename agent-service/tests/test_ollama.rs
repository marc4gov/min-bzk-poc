use autoagents::core::agent::memory::SlidingWindowMemory;
use autoagents::core::agent::prebuilt::executor::{ReActAgent, ReActAgentOutput};
use autoagents::core::agent::task::Task;
use autoagents::core::agent::{AgentBuilder, DirectAgent};
use autoagents::llm::backends::ollama::Ollama;
use autoagents::llm::builder::LLMBuilder;
use autoagents::prelude::AgentOutputT;
use autoagents_derive::{agent, AgentHooks, AgentOutput};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Smoke test: local Ollama reachable and AutoAgents can complete a tiny turn.
#[derive(Debug, Serialize, Deserialize, AgentOutput)]
struct PingOut {
    #[output(description = "Antwoord")]
    answer: String,
}

impl From<ReActAgentOutput> for PingOut {
    fn from(output: ReActAgentOutput) -> Self {
        PingOut {
            answer: output.response,
        }
    }
}

#[agent(
    name = "ping",
    description = "Antwoord kort met alleen het woord 'pong'.",
    output = PingOut,
)]
#[derive(Default, Clone, AgentHooks)]
struct PingAgent {}

#[tokio::test]
#[ignore]
async fn test_ollama_ping_agent() {
    let llm: Arc<Ollama> = LLMBuilder::<Ollama>::new()
        .base_url(
            std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".into()),
        )
        .model(std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen3-coder:latest".into()))
        .build()
        .expect("Ollama LLM build");

    let ping = PingAgent::default();
    let handle = AgentBuilder::<_, DirectAgent>::new(ReActAgent::new(ping))
        .llm(llm)
        .memory(Box::new(SlidingWindowMemory::new(4)))
        .build()
        .await
        .expect("agent build");

    let out: PingOut = handle
        .agent
        .run(Task::new("Reply with only: pong"))
        .await
        .expect("agent run");

    assert!(!out.answer.to_lowercase().contains("error"));
}
