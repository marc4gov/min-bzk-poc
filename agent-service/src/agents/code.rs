use std::sync::Arc;

pub struct CodeAgent {
    llm: Arc<dyn crate::llm::LLMProvider>,
}

impl CodeAgent {
    pub fn new(llm: Arc<dyn crate::llm::LLMProvider>) -> Self {
        Self { llm }
    }

    pub async fn process(&self, task: &str) -> anyhow::Result<String> {
        let prompt = format!(
            "Je bent een code expert. Schrijf duidelijke, goed gedocumenteerde code.\n\nVraag: {}",
            task
        );

        let response = self.llm.generate(crate::llm::LLMRequest {
            prompt,
            max_tokens: Some(1000),
            temperature: Some(0.2),
        }).await?;

        Ok(response.content)
    }
}
