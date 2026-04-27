use std::sync::Arc;

pub struct SchrijfAgent {
    llm: Arc<dyn crate::llm::LLMProvider>,
}

impl SchrijfAgent {
    pub fn new(llm: Arc<dyn crate::llm::LLMProvider>) -> Self {
        Self { llm }
    }

    pub async fn process(&self, task: &str) -> anyhow::Result<String> {
        let prompt = format!(
            "Je bent een behulpzame schrijfassistant. Schrijf in het Nederlands.\n\nVraag: {}",
            task
        );

        let response = self.llm.generate(crate::llm::LLMRequest {
            prompt,
            max_tokens: Some(1000),
            temperature: Some(0.7),
        }).await?;

        Ok(response.content)
    }
}
