use localassistant_lib::services::{StorageService, ChatHistory, Template};

#[tokio::test]
async fn test_create_chat_history() {
    let test_db = "/tmp/test_localassistant_create.db";
    let _ = std::fs::remove_file(test_db);

    let service = StorageService::new(test_db.into()).await.unwrap();
    let history: ChatHistory = service.create_chat_history("Test Chat").await.unwrap();
    assert_eq!(history.title, "Test Chat");
    assert!(history.id.len() > 0);
    assert!(history.messages.is_empty());
    let _ = std::fs::remove_file(test_db);
}

#[tokio::test]
async fn test_add_message() {
    let test_db = "/tmp/test_localassistant_add.db";
    let _ = std::fs::remove_file(test_db);

    let service = StorageService::new(test_db.into()).await.unwrap();
    let history = service.create_chat_history("Test Chat").await.unwrap();
    let message = service.add_chat_message(&history.id, "user", "Hello").await.unwrap();

    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello");

    let loaded = service.get_chat_history(&history.id).await.unwrap().unwrap();
    assert_eq!(loaded.messages.len(), 1);

    let _ = std::fs::remove_file(test_db);
}

#[tokio::test]
async fn test_get_histories() {
    let test_db = "/tmp/test_localassistant_list.db";
    let _ = std::fs::remove_file(test_db);

    let service = StorageService::new(test_db.into()).await.unwrap();
    service.create_chat_history("Chat 1").await.unwrap();
    service.create_chat_history("Chat 2").await.unwrap();

    let histories = service.get_chat_histories().await.unwrap();
    assert_eq!(histories.len(), 2);

    let _ = std::fs::remove_file(test_db);
}

#[tokio::test]
async fn test_delete_history() {
    let test_db = "/tmp/test_localassistant_delete.db";
    let _ = std::fs::remove_file(test_db);

    let service = StorageService::new(test_db.into()).await.unwrap();
    let history = service.create_chat_history("To Delete").await.unwrap();
    service.delete_chat_history(&history.id).await.unwrap();

    let result = service.get_chat_history(&history.id).await.unwrap();
    assert!(result.is_none());

    let _ = std::fs::remove_file(test_db);
}

#[tokio::test]
async fn test_templates_seeded() {
    let test_db = "/tmp/test_localassistant_templates.db";
    let _ = std::fs::remove_file(test_db);

    let service = StorageService::new(test_db.into()).await.unwrap();
    let templates: Vec<Template> = service.get_templates().await.unwrap();
    assert!(templates.len() >= 3);
    assert!(templates.iter().any(|t| t.is_builtin));

    let _ = std::fs::remove_file(test_db);
}

#[tokio::test]
async fn test_create_template() {
    let test_db = "/tmp/test_localassistant_create_template.db";
    let _ = std::fs::remove_file(test_db);

    let service = StorageService::new(test_db.into()).await.unwrap();
    let template: Template = service.create_template(
        "Custom",
        "My template",
        "Test prompt: {{var}}"
    ).await.unwrap();

    assert_eq!(template.name, "Custom");
    assert_eq!(template.is_builtin, false);

    let _ = std::fs::remove_file(test_db);
}
