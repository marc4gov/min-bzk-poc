use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentEvent {
    Started(AgentStartedEvent),
    Progress(AgentProgressEvent),
    ToolCall(ToolCallEvent),
    Complete(AgentCompleteEvent),
    Error(AgentErrorEvent),
    OrchestrationComplete(OrchestrationCompleteEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStartedEvent {
    pub agent_id: String,
    pub agent_type: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProgressEvent {
    pub agent_id: String,
    pub delta: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
    pub agent_id: String,
    pub tool_name: String,
    pub args: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCompleteEvent {
    pub agent_id: String,
    pub result: String,
    pub tokens_used: Option<u32>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentErrorEvent {
    pub agent_id: String,
    pub error: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationCompleteEvent {
    pub final_response: String,
    pub agents_used: Vec<String>,
    pub total_tokens: u32,
    pub duration_ms: u64,
    pub timestamp: i64,
}

impl AgentEvent {
    pub fn agent_started(agent_id: String, agent_type: String) -> Self {
        Self::Started(AgentStartedEvent {
            agent_id,
            agent_type,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    pub fn agent_progress(agent_id: String, delta: String) -> Self {
        Self::Progress(AgentProgressEvent {
            agent_id,
            delta,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    pub fn tool_call(agent_id: String, tool_name: String, args: serde_json::Value) -> Self {
        Self::ToolCall(ToolCallEvent {
            agent_id,
            tool_name,
            args,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    pub fn agent_complete(agent_id: String, result: String, tokens_used: Option<u32>) -> Self {
        Self::Complete(AgentCompleteEvent {
            agent_id,
            result,
            tokens_used,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
}
