use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info};
use uuid::Uuid;

use nairi_core::config::AppConfig;

#[derive(Debug)]
pub struct DockerEngine {
    workspace_root: PathBuf,
}

impl DockerEngine {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    pub async fn run_static_analysis(
        &self,
        run_id: Uuid,
        config: &AppConfig,
        prompt_content: &str,
        apk_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let run_workspace = self.workspace_root.join(run_id.to_string());
        tokio::fs::create_dir_all(&run_workspace).await?;
        let reports_dir = run_workspace.join("reports");
        tokio::fs::create_dir_all(&reports_dir).await?;

        // Save prompt for auditing
        let prompt_path = run_workspace.join("prompt.txt");
        tokio::fs::write(&prompt_path, prompt_content).await?;

        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--rm")
            .arg("-e")
            .arg(format!("GEMINI_API_KEY={}", config.api_key))
            .arg("-e")
            .arg(format!("GOOGLE_GEMINI_BASE_URL={}", config.base_url))
            .arg("-v")
            .arg(format!("{}:/workspace/target.apk:ro", apk_path.display()))
            .arg("-v")
            .arg(format!("{}:/workspace/reports", reports_dir.display()))
            .arg(&config.static_analysis_image)
            .arg("gemini")
            .arg("--debug")
            .arg("--model")
            .arg(&config.model_name)
            .arg("--yolo")
            .arg("--prompt")
            .arg(prompt_content)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        info!("Spawning Docker container for run {}", run_id);

        let mut child = cmd.spawn()?;
        let status = child.wait().await?;

        if status.success() {
            info!("Static analysis completed successfully for run {}", run_id);
            Ok(())
        } else {
            error!("Static analysis failed for run {}", run_id);
            Err("Container exited with non-zero status".into())
        }
    }
}
