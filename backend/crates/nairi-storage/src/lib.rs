use chrono::{DateTime, Utc};
use nairi_core::analysis::{AnalysisRun, AnalysisStatus};
use nairi_core::config::{AppConfig, PromptConfig};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Row, Sqlite};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS system_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                model_name TEXT NOT NULL,
                api_key TEXT NOT NULL,
                base_url TEXT NOT NULL,
                static_analysis_image TEXT NOT NULL,
                runtime_analysis_image TEXT NOT NULL
            );
        ",
        )
            .execute(&pool)
            .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS prompts (
                name TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                updated_at DATETIME NOT NULL
            );
        ",
        )
            .execute(&pool)
            .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS runs (
                id TEXT PRIMARY KEY,
                package_name TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            );
        ",
        )
            .execute(&pool)
            .await?;

        let default_config = AppConfig::default();
        sqlx::query(
            "
            INSERT OR IGNORE INTO system_config (id, model_name, api_key, base_url, static_analysis_image, runtime_analysis_image)
            VALUES (1, ?, ?, ?, ?, ?)
        ",
        )
            .bind(&default_config.model_name)
            .bind(&default_config.api_key)
            .bind(&default_config.base_url)
            .bind(&default_config.static_analysis_image)
            .bind(&default_config.runtime_analysis_image)
            .execute(&pool)
            .await?;

        let default_prompt = include_str!("../../../../sample_prompt.txt");
        sqlx::query(
            "
            INSERT OR IGNORE INTO prompts (name, content, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP)
        ",
        )
            .bind("static_analysis")
            .bind(default_prompt)
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub async fn get_config(&self) -> AppConfig {
        let row =
            sqlx::query("SELECT model_name, api_key, base_url, static_analysis_image, runtime_analysis_image FROM system_config WHERE id = 1")
                .fetch_one(&self.pool)
                .await
                .expect("system_config row should always exist");

        AppConfig {
            model_name: row.get("model_name"),
            api_key: row.get("api_key"),
            base_url: row.get("base_url"),
            static_analysis_image: row.get("static_analysis_image"),
            runtime_analysis_image: row.get("runtime_analysis_image"),
        }
    }

    pub async fn update_config(&self, new_config: AppConfig) {
        sqlx::query(
            "
            UPDATE system_config 
            SET model_name = ?, api_key = ?, base_url = ?, static_analysis_image = ?, runtime_analysis_image = ?
            WHERE id = 1
        ",
        )
            .bind(&new_config.model_name)
            .bind(&new_config.api_key)
            .bind(&new_config.base_url)
            .bind(&new_config.static_analysis_image)
            .bind(&new_config.runtime_analysis_image)
            .execute(&self.pool)
            .await
            .expect("failed to update config");
    }

    pub async fn get_prompt(&self, name: &str) -> Option<PromptConfig> {
        let row = sqlx::query("SELECT name, content FROM prompts WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        Some(PromptConfig {
            name: row.get("name"),
            content: row.get("content"),
        })
    }

    pub async fn update_prompt(&self, prompt: PromptConfig) {
        sqlx::query("
            INSERT INTO prompts (name, content, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(name) DO UPDATE SET content = excluded.content, updated_at = CURRENT_TIMESTAMP
        ")
            .bind(&prompt.name)
            .bind(&prompt.content)
            .execute(&self.pool)
            .await
            .expect("failed to update prompt");
    }

    pub async fn insert_run(&self, run: AnalysisRun) {
        let status_str = format!("{:?}", run.status);
        sqlx::query(
            "
            INSERT INTO runs (id, package_name, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
        ",
        )
            .bind(run.id.to_string())
            .bind(&run.package_name)
            .bind(status_str)
            .bind(run.created_at)
            .bind(run.updated_at)
            .execute(&self.pool)
            .await
            .expect("failed to insert run");
    }

    pub async fn get_run(&self, id: Uuid) -> Option<AnalysisRun> {
        let row = sqlx::query(
            "SELECT id, package_name, status, created_at, updated_at FROM runs WHERE id = ?",
        )
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let id_str: String = row.get("id");
        let status_str: String = row.get("status");

        // This is a bit hacky, normally you'd implement sqlx::Type for the enum
        let status = match status_str.as_str() {
            "Queued" => AnalysisStatus::Queued,
            "Running" => AnalysisStatus::Running,
            "Completed" => AnalysisStatus::Completed,
            "Failed" => AnalysisStatus::Failed,
            _ => return None,
        };

        let created_str: String = row.get("created_at");
        let updated_str: String = row.get("updated_at");

        Some(AnalysisRun {
            id: Uuid::from_str(&id_str).unwrap(),
            package_name: row.get("package_name"),
            status,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .unwrap_or_else(|_| Utc::now().into())
                .into(),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .unwrap_or_else(|_| Utc::now().into())
                .into(),
        })
    }

    pub async fn list_runs(&self) -> Vec<AnalysisRun> {
        let rows = sqlx::query(
            "SELECT id, package_name, status, created_at, updated_at FROM runs ORDER BY created_at DESC",
        )
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        rows.into_iter()
            .filter_map(|row| {
                let id_str: String = row.get("id");
                let status_str: String = row.get("status");

                let status = match status_str.as_str() {
                    "Queued" | "queued" => AnalysisStatus::Queued,
                    "Running" | "running" => AnalysisStatus::Running,
                    "Completed" | "completed" => AnalysisStatus::Completed,
                    "Failed" | "failed" => AnalysisStatus::Failed,
                    _ => return None,
                };

                let created_str: String = row.get("created_at");
                let updated_str: String = row.get("updated_at");

                Some(AnalysisRun {
                    id: Uuid::from_str(&id_str).unwrap_or_default(),
                    package_name: row.get("package_name"),
                    status,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .into(),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .into(),
                })
            })
            .collect()
    }

    pub async fn update_run(&self, run: AnalysisRun) {
        let status_str = format!("{:?}", run.status);
        sqlx::query(
            "
            UPDATE runs 
            SET status = ?, updated_at = ?
            WHERE id = ?
        ",
        )
            .bind(status_str)
            .bind(run.updated_at)
            .bind(run.id.to_string())
            .execute(&self.pool)
            .await
            .expect("failed to update run");
    }
}
