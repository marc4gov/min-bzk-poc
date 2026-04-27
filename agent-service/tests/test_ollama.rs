use agent_service::llm::{LLMProvider, OllamaProvider, LLMRequest};

#[tokio::test]
#[ignore]
async fn test_ollama_generate() {
    let provider = OllamaProvider::new();
    if !provider.is_available().await {
        println!("Ollama not available, skipping test");
        return;
    }
    let request = LLMRequest {
        prompt: "Say 'Hello, Ollama!' in Dutch.".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.7),
    };
    let response = provider.generate(request).await.unwrap();
    assert!(!response.content.is_empty());
    println!("Ollama response: {}", response.content);
}
