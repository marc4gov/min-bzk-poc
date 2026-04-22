use crate::errors::Result;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub metadata: String,
    pub created_at: i64,
}

pub struct StorageService {
    pool: SqlitePool,
}

impl StorageService {
    pub async fn new(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Create database if it doesn't exist
        if !db_path.exists() {
            tokio::fs::File::create(db_path).await?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        let service = Self { pool };
        service.init_schema().await?;
        service.seed_templates().await?;

        Ok(service)
    }

    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
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
                content TEXT NOT NULL,
                category TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn seed_templates(&self) -> Result<()> {
        // Check if templates already exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates")
            .fetch_one(&self.pool)
            .await?;

        if count > 0 {
            return Ok(());
        }

        let templates = vec![
            Template {
                id: Uuid::new_v4().to_string(),
                name: "Email opstellen".to_string(),
                description: "Opstel sjabloon voor professionele e-mails".to_string(),
                content: "Help me een professionele e-mail te schrijven over:\n\n{{onderwerp}}\n\nDetails:\n{{details}}\n\nToon: {{toon}}".to_string(),
                category: "productivity".to_string(),
            },
            Template {
                id: Uuid::new_v4().to_string(),
                name: "Samenvatting".to_string(),
                description: "Maak een samenvatting van lange teksten".to_string(),
                content: "Maak een samenvatting van de volgende tekst:\n\n{{tekst}}\n\nLengte: {{lengte}}\nFocus op: {{focus}}".to_string(),
                category: "productivity".to_string(),
            },
            Template {
                id: Uuid::new_v4().to_string(),
                name: "Code uitleg".to_string(),
                description: "Leg code uit in eenvoudige termen".to_string(),
                content: "Leg de volgende code uit:\n\n```\n{{code}}\n```\n\nTaal: {{taal}}\nNiveau: {{niveau}}".to_string(),
                category: "developer".to_string(),
            },
        ];

        for template in &templates {
            sqlx::query(
                "INSERT INTO templates (id, name, description, content, category) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&template.id)
            .bind(&template.name)
            .bind(&template.description)
            .bind(&template.content)
            .bind(&template.category)
            .execute(&self.pool)
            .await?;
        }

        tracing::info!("Seeded {} templates", templates.len());
        Ok(())
    }

    pub async fn create_conversation(&self, title: &str) -> Result<Conversation> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        sqlx::query("INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(title)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(Conversation {
            id,
            title: title.to_string(),
            messages: vec![],
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut conversations = Vec::new();
        for (id, title, created_at, updated_at) in rows {
            let messages = self.get_messages(&id).await?;
            conversations.push(Conversation {
                id,
                title,
                messages,
                created_at,
                updated_at,
            });
        }

        Ok(conversations)
    }

    pub async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>> {
        let row = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT id, title, created_at, updated_at FROM conversations WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, title, created_at, updated_at)) = row {
            let messages = self.get_messages(&id).await?;
            Ok(Some(Conversation {
                id,
                title,
                messages,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn add_message(&self, conversation_id: &str, role: &str, content: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, timestamp) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .bind(now)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(conversation_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_messages(&self, conversation_id: &str) -> Result<Vec<ChatMessage>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64)>(
            "SELECT id, role, content, timestamp FROM messages WHERE conversation_id = ? ORDER BY timestamp ASC"
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, role, content, timestamp)| ChatMessage {
                id,
                role,
                content,
                timestamp,
            })
            .collect())
    }

    pub async fn delete_conversation(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_templates(&self) -> Result<Vec<Template>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
            "SELECT id, name, description, content, category FROM templates ORDER BY category, name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, name, description, content, category)| Template {
                id,
                name,
                description,
                content,
                category,
            })
            .collect())
    }

    pub async fn save_document(&self, title: &str, content: &str, metadata: &str) -> Result<Document> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO documents (id, title, content, metadata, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(title)
        .bind(content)
        .bind(metadata)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(Document {
            id,
            title: title.to_string(),
            content: content.to_string(),
            metadata: metadata.to_string(),
            created_at: now,
        })
    }

    pub async fn get_documents(&self) -> Result<Vec<Document>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, i64)>(
            "SELECT id, title, content, metadata, created_at FROM documents ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, title, content, metadata, created_at)| Document {
                id,
                title,
                content,
                metadata,
                created_at,
            })
            .collect())
    }

    pub async fn delete_document(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM documents WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
