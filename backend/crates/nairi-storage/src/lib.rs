use std::collections::HashMap;
use std::sync::Arc;

use nairi_core::analysis::AnalysisRun;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Storage {
    inner: Arc<RwLock<HashMap<Uuid, AnalysisRun>>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert_run(&self, run: AnalysisRun) {
        self.inner.write().await.insert(run.id, run);
    }

    pub async fn get_run(&self, id: Uuid) -> Option<AnalysisRun> {
        self.inner.read().await.get(&id).cloned()
    }

    pub async fn update_run(&self, run: AnalysisRun) {
        self.inner.write().await.insert(run.id, run);
    }
}
