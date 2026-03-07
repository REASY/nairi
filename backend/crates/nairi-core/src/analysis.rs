use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRun {
    pub id: Uuid,
    pub package_name: String,
    pub status: AnalysisStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AnalysisRun {
    pub fn new(package_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            package_name,
            status: AnalysisStatus::Queued,
            created_at: now,
            updated_at: now,
        }
    }
}
