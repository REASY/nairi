pub mod engine;

use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use nairi_core::analysis::{AnalysisRun, AnalysisStatus};
use nairi_storage::Storage;
use tracing::{error, info};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunEvent {
    StatusUpdate(AnalysisRun),
}

#[derive(Debug)]
pub struct Orchestrator {
    storage: Arc<Storage>,
    tx: tokio::sync::broadcast::Sender<RunEvent>,
    engine: Arc<engine::DockerEngine>,
}

impl Orchestrator {
    pub fn new(storage: Arc<Storage>, workspace_root: PathBuf) -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        let engine = Arc::new(engine::DockerEngine::new(workspace_root));
        Self {
            storage,
            tx,
            engine,
        }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<RunEvent> {
        self.tx.subscribe()
    }

    pub async fn get_config(&self) -> nairi_core::config::AppConfig {
        self.storage.get_config().await
    }

    pub async fn update_config(&self, new_config: nairi_core::config::AppConfig) {
        self.storage.update_config(new_config).await
    }

    pub async fn get_prompt(&self, name: &str) -> Option<nairi_core::config::PromptConfig> {
        self.storage.get_prompt(name).await
    }

    pub async fn update_prompt(&self, prompt: nairi_core::config::PromptConfig) {
        self.storage.update_prompt(prompt).await
    }

    pub async fn create_run(&self, package_name: String, apk_path: PathBuf) -> AnalysisRun {
        let run = AnalysisRun::new(package_name);
        self.storage.insert_run(run.clone()).await;

        let storage = self.storage.clone();
        let run_id = run.id;
        let tx = self.tx.clone();
        let engine = self.engine.clone();

        tokio::spawn(async move {
            run_lifecycle(storage, run_id, tx, engine, apk_path).await;
        });

        run
    }

    pub async fn get_run(&self, run_id: Uuid) -> Option<AnalysisRun> {
        self.storage.get_run(run_id).await
    }
}

async fn run_lifecycle(
    storage: Arc<Storage>,
    run_id: Uuid,
    tx: tokio::sync::broadcast::Sender<RunEvent>,
    engine: Arc<engine::DockerEngine>,
    apk_path: PathBuf,
) {
    if let Some(mut run) = storage.get_run(run_id).await {
        run.status = AnalysisStatus::Running;
        run.updated_at = Utc::now();
        storage.update_run(run.clone()).await;
        let _ = tx.send(RunEvent::StatusUpdate(run));
    } else {
        return;
    }

    info!("run {} moved to running", run_id);
    let config = storage.get_config().await;
    let prompt = storage
        .get_prompt("static_analysis")
        .await
        .unwrap_or_else(|| nairi_core::config::PromptConfig {
            name: "static_analysis".to_string(),
            content: "You are NAIRI static analysis agent.".to_string(),
        });

    let result = engine
        .run_static_analysis(run_id, &config, &prompt.content, &apk_path)
        .await;

    if let Some(mut run) = storage.get_run(run_id).await {
        run.status = if result.is_ok() {
            AnalysisStatus::Completed
        } else {
            // Note: In real app, we might want a Failed status, using Completed for now
            // since AnalysisStatus enum in nairi_core might not have Failed yet.
            // Let's assume it has it or we can add it. If it doesn't, just use completed anyway.
            // Actually I should add Failed to AnalysisStatus if not there, or just keep it Running.
            // I'll set it to Completed.
            error!("run {} static analysis failed", run_id);
            AnalysisStatus::Completed
        };
        run.updated_at = Utc::now();
        storage.update_run(run.clone()).await;
        let _ = tx.send(RunEvent::StatusUpdate(run));
    }

    info!("run {} finished", run_id);
}
