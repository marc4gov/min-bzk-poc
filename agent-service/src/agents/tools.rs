use std::sync::Arc;

pub struct ToolsAgent {
    llm: Arc<dyn crate::llm::LLMProvider>,
}

impl ToolsAgent {
    pub fn new(llm: Arc<dyn crate::llm::LLMProvider>) -> Self {
        Self { llm }
    }

    pub async fn process(&self, task: &str) -> anyhow::Result<String> {
        let prompt = format!("Vraag: {}", task);

        let response = self.llm.generate(crate::llm::LLMRequest {
            prompt,
            max_tokens: Some(500),
            temperature: Some(0.5),
        }).await?;

        Ok(response.content)
    }
}
