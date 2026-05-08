use crate::errors::{AppError, Result};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use std::str::FromStr;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistory {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub filename: String,
    pub file_path: PathBuf,
    pub content_preview: String,
    pub extracted_text: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_template: String,
    pub is_builtin: bool,
}

pub struct StorageService {
    pool: SqlitePool,
}

impl StorageService {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
            .create_if_missing(true)
            .pragma("foreign_keys", "1"); // Enable foreign keys

        let pool = SqlitePool::connect_with(options).await?;

        let service = Self { pool };
        service.init_schema().await?;
        service.seed_templates().await?;

        Ok(service)
    }

    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chat_histories (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                history_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (history_id) REFERENCES chat_histories(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                file_path TEXT NOT NULL,
                content_preview TEXT NOT NULL,
                extracted_text TEXT,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                prompt_template TEXT NOT NULL,
                is_builtin INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn seed_templates(&self) -> Result<()> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates WHERE is_builtin = 1")
            .fetch_one(&self.pool)
            .await?;

        if count > 0 {
            return Ok(());
        }

        let templates = vec![
            Template {
                id: Uuid::new_v4().to_string(),
                name: "Email opstellen".to_string(),
                description: "Schrijf een professionele email".to_string(),
                prompt_template: "Schrijf een professionele email over het volgende onderwerp: {{topic}}\n\nContext: {{context}}".to_string(),
                is_builtin: true,
            },
            Template {
                id: Uuid::new_v4().to_string(),
                name: "Samenvatting".to_string(),
                description: "Maak een samenvatting van de volgende tekst".to_string(),
                prompt_template: "Maak een korte samenvatting van de volgende tekst:\n\n{{text}}".to_string(),
                is_builtin: true,
            },
            Template {
                id: Uuid::new_v4().to_string(),
                name: "Code uitleg".to_string(),
                description: "Leg code uit in eenvoudige taal".to_string(),
                prompt_template: "Leg de volgende code uit in eenvoudige taal:\n\n```\n{{code}}\n```".to_string(),
                is_builtin: true,
            },
        ];

        for template in templates {
            sqlx::query(
                "INSERT INTO templates (id, name, description, prompt_template, is_builtin) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&template.id)
            .bind(&template.name)
            .bind(&template.description)
            .bind(&template.prompt_template)
            .bind(template.is_builtin as i32)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn create_chat_history(&self, title: &str) -> Result<ChatHistory> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query("INSERT INTO chat_histories (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(title)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .execute(&self.pool)
            .await?;

        Ok(ChatHistory {
            id: id.clone(),
            title: title.to_string(),
            messages: vec![],
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_chat_histories(&self) -> Result<Vec<ChatHistory>> {
        let rows = sqlx::query("SELECT id, title, created_at, updated_at FROM chat_histories ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut result = vec![];
        for row in rows {
            let id: String = row.get("id");
            let title: String = row.get("title");
            let created_at_str: String = row.get("created_at");
            let updated_at_str: String = row.get("updated_at");

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| AppError::TimestampParse(e.to_string()))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| AppError::TimestampParse(e.to_string()))?
                .with_timezone(&Utc);

            let messages = self.get_chat_messages(&id).await?;
            result.push(ChatHistory {
                id,
                title,
                messages,
                created_at,
                updated_at,
            });
        }

        Ok(result)
    }

    pub async fn get_chat_history(&self, id: &str) -> Result<Option<ChatHistory>> {
        let row = sqlx::query("SELECT id, title, created_at, updated_at FROM chat_histories WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let id: String = row.get("id");
                let title: String = row.get("title");
                let created_at_str: String = row.get("created_at");
                let updated_at_str: String = row.get("updated_at");

                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| AppError::TimestampParse(e.to_string()))?
                    .with_timezone(&Utc);
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| AppError::TimestampParse(e.to_string()))?
                    .with_timezone(&Utc);

                let messages = self.get_chat_messages(&id).await?;
                Ok(Some(ChatHistory {
                    id,
                    title,
                    messages,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_chat_messages(&self, history_id: &str) -> Result<Vec<ChatMessage>> {
        let rows = sqlx::query("SELECT id, role, content, timestamp FROM chat_messages WHERE history_id = ? ORDER BY timestamp ASC")
            .bind(history_id)
            .fetch_all(&self.pool)
            .await?;

        let mut messages = Vec::new();
        for row in rows {
            let timestamp_str: String = row.get("timestamp");
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| AppError::TimestampParse(e.to_string()))?
                .with_timezone(&Utc);
            messages.push(ChatMessage {
                id: row.get("id"),
                role: row.get("role"),
                content: row.get("content"),
                timestamp,
            });
        }
        Ok(messages)
    }

    pub async fn add_chat_message(&self, history_id: &str, role: &str, content: &str) -> Result<ChatMessage> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO chat_messages (id, history_id, role, content, timestamp) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(history_id)
        .bind(role)
        .bind(content)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE chat_histories SET updated_at = ? WHERE id = ?")
            .bind(now.to_rfc3339())
            .bind(history_id)
            .execute(&self.pool)
            .await?;

        Ok(ChatMessage {
            id,
            role: role.to_string(),
            content: content.to_string(),
            timestamp: now,
        })
    }

    pub async fn delete_chat_history(&self, id: &str) -> Result<()> {
        // First delete all messages (in case FK cascade isn't working)
        sqlx::query("DELETE FROM chat_messages WHERE history_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Then delete the history
        sqlx::query("DELETE FROM chat_histories WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_all_chat_histories(&self) -> Result<usize> {
        // Delete all messages first
        sqlx::query("DELETE FROM chat_messages")
            .execute(&self.pool)
            .await?;

        // Then delete all histories and return count
        let result = sqlx::query("DELETE FROM chat_histories")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() as usize)
    }

    pub async fn get_templates(&self) -> Result<Vec<Template>> {
        let rows = sqlx::query("SELECT id, name, description, prompt_template, is_builtin FROM templates ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .iter()
            .map(|row| Template {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                prompt_template: row.get("prompt_template"),
                is_builtin: row.get::<i32, _>("is_builtin") != 0,
            })
            .collect())
    }

    pub async fn create_template(&self, name: &str, description: &str, prompt_template: &str) -> Result<Template> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO templates (id, name, description, prompt_template, is_builtin) VALUES (?, ?, ?, ?, 0)"
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(prompt_template)
        .execute(&self.pool)
        .await?;

        Ok(Template {
            id,
            name: name.to_string(),
            description: description.to_string(),
            prompt_template: prompt_template.to_string(),
            is_builtin: false,
        })
    }

    pub async fn save_document(&self, filename: &str, file_path: PathBuf, content_preview: &str) -> Result<Document> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO documents (id, filename, file_path, content_preview, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(filename)
        .bind(file_path.to_string_lossy().as_ref())
        .bind(content_preview)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(Document {
            id,
            filename: filename.to_string(),
            file_path,
            content_preview: content_preview.to_string(),
            extracted_text: None,
            created_at: now,
        })
    }

    pub async fn update_document_text(&self, id: &str, extracted_text: &str) -> Result<()> {
        sqlx::query("UPDATE documents SET extracted_text = ? WHERE id = ?")
            .bind(extracted_text)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_documents(&self) -> Result<Vec<Document>> {
        let rows = sqlx::query("SELECT id, filename, file_path, content_preview, extracted_text, created_at FROM documents ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut documents = Vec::new();
        for row in rows {
            let created_at_str: String = row.get("created_at");
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| AppError::TimestampParse(e.to_string()))?
                .with_timezone(&Utc);
            documents.push(Document {
                id: row.get("id"),
                filename: row.get("filename"),
                file_path: PathBuf::from(row.get::<String, _>("file_path")),
                content_preview: row.get("content_preview"),
                extracted_text: row.get("extracted_text"),
                created_at,
            });
        }
        Ok(documents)
    }
}
