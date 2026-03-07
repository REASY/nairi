use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use nairi_core::analysis::{AnalysisRun, AnalysisStatus};
use nairi_storage::Storage;
use tokio::time::sleep;
use tracing::info;
use uuid::Uuid;

#[derive(Debug)]
pub struct Orchestrator {
    storage: Arc<Storage>,
}

impl Orchestrator {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub async fn create_run(&self, package_name: String) -> AnalysisRun {
        let run = AnalysisRun::new(package_name);
        self.storage.insert_run(run.clone()).await;

        let storage = self.storage.clone();
        let run_id = run.id;
        tokio::spawn(async move {
            simulate_run_lifecycle(storage, run_id).await;
        });

        run
    }

    pub async fn get_run(&self, run_id: Uuid) -> Option<AnalysisRun> {
        self.storage.get_run(run_id).await
    }
}

async fn simulate_run_lifecycle(storage: Arc<Storage>, run_id: Uuid) {
    if let Some(mut run) = storage.get_run(run_id).await {
        run.status = AnalysisStatus::Running;
        run.updated_at = Utc::now();
        storage.update_run(run).await;
    } else {
        return;
    }

    info!("run {} moved to running", run_id);
    sleep(Duration::from_secs(5)).await;

    if let Some(mut run) = storage.get_run(run_id).await {
        run.status = AnalysisStatus::Completed;
        run.updated_at = Utc::now();
        storage.update_run(run).await;
    } else {
        return;
    }

    info!("run {} moved to completed", run_id);
}
