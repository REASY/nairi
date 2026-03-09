pub mod engine;
pub mod pipeline;

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
        let package_name = run.package_name.clone();
        let tx = self.tx.clone();
        let engine = self.engine.clone();

        tokio::spawn(async move {
            run_lifecycle(storage, run_id, package_name, tx, engine, apk_path).await;
        });

        run
    }

    pub async fn get_run(&self, run_id: Uuid) -> Option<AnalysisRun> {
        self.storage.get_run(run_id).await
    }

    pub async fn list_runs(&self) -> Vec<AnalysisRun> {
        self.storage.list_runs().await
    }

    pub async fn get_report(&self, run_id: Uuid) -> Option<String> {
        self.engine.get_report(run_id).await
    }
}

async fn run_lifecycle(
    storage: Arc<Storage>,
    run_id: Uuid,
    package_name: String,
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

    let static_result = engine
        .run_static_analysis(run_id, &config, &prompt.content, &apk_path)
        .await;
    if static_result.is_err() {
        error!("run {} static analysis failed", run_id);
    }

    let runtime_prompt = storage
        .get_prompt("runtime_analysis")
        .await
        .unwrap_or_else(|| nairi_core::config::PromptConfig {
            name: "runtime_analysis".to_string(),
            content: "You are NAIRI runtime analysis agent.".to_string(),
        });

    let runtime_result = if static_result.is_ok() {
        engine
            .run_runtime_analysis(
                run_id,
                &config,
                &runtime_prompt.content,
                &package_name,
                &apk_path,
            )
            .await
    } else {
        Err("runtime skipped because static analysis failed".into())
    };
    if runtime_result.is_err() {
        error!("run {} runtime analysis failed", run_id);
    }

    if let Some(mut run) = storage.get_run(run_id).await {
        run.status = if static_result.is_ok() && runtime_result.is_ok() {
            AnalysisStatus::Completed
        } else {
            AnalysisStatus::Failed
        };
        run.updated_at = Utc::now();
        storage.update_run(run.clone()).await;
        let _ = tx.send(RunEvent::StatusUpdate(run));
    }

    info!("run {} finished", run_id);
}
