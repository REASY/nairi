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
            .arg("--model")
            .arg(&config.model_name)
            .arg("--yolo")
            .arg("--prompt")
            .arg(prompt_content)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        info!("Spawning static analysis container for run {}", run_id);
        let output = cmd.output().await?;
        if output.status.success() {
            info!("Static analysis completed successfully for run {}", run_id);
            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                "Static analysis failed for run {}. stdout: {} stderr: {}",
                run_id, stdout, stderr
            );
            Err(std::io::Error::other(format!(
                "static analysis container failed with status {}",
                output.status
            ))
                .into())
        }
    }

    pub async fn run_runtime_analysis(
        &self,
        run_id: Uuid,
        config: &AppConfig,
        prompt_content: &str,
        package_name: &str,
        apk_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let run_workspace = self.workspace_root.join(run_id.to_string());
        tokio::fs::create_dir_all(&run_workspace).await?;
        let reports_dir = run_workspace.join("reports");
        tokio::fs::create_dir_all(&reports_dir).await?;

        let runtime_prompt =
            build_runtime_prompt(prompt_content, package_name, &config.adb_connection_string);
        let prompt_path = run_workspace.join("runtime-prompt.txt");
        tokio::fs::write(&prompt_path, &runtime_prompt).await?;

        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--rm")
            .arg("--add-host")
            .arg("host.docker.internal:host-gateway")
            .arg("-e")
            .arg(format!("GEMINI_API_KEY={}", config.api_key))
            .arg("-e")
            .arg(format!("GOOGLE_GEMINI_BASE_URL={}", config.base_url))
            .arg("-e")
            .arg(format!(
                "ADB_CONNECTION_STRING={}",
                config.adb_connection_string
            ))
            .arg("-v")
            .arg(format!("{}:/workspace/target.apk:ro", apk_path.display()))
            .arg("-v")
            .arg(format!("{}:/workspace/reports", reports_dir.display()))
            .arg(&config.runtime_analysis_image)
            .arg("gemini")
            .arg("--model")
            .arg(&config.model_name)
            .arg("--yolo")
            .arg("--prompt")
            .arg(&runtime_prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        info!("Spawning runtime analysis container for run {}", run_id);
        let output = cmd.output().await?;
        if output.status.success() {
            info!("Runtime analysis completed successfully for run {}", run_id);
            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                "Runtime analysis failed for run {}. stdout: {} stderr: {}",
                run_id, stdout, stderr
            );
            Err(std::io::Error::other(format!(
                "runtime analysis container failed with status {}",
                output.status
            ))
                .into())
        }
    }

    pub async fn get_report(&self, run_id: Uuid) -> Option<String> {
        let reports_dir = self.workspace_root.join(run_id.to_string()).join("reports");
        let report_candidates = [
            "analysis-report.md",
            "runtime-analysis-report.md",
            "static-analysis-report.md",
        ];
        for candidate in report_candidates {
            let report = reports_dir.join(candidate);
            if report.exists()
                && let Ok(content) = tokio::fs::read_to_string(&report).await
            {
                return Some(content);
            }
        }

        // Fallback to any markdown file
        let mut entries = tokio::fs::read_dir(&reports_dir).await.ok()?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                return tokio::fs::read_to_string(path).await.ok();
            }
        }
        None
    }
}

fn build_runtime_prompt(
    base_prompt: &str,
    package_name: &str,
    adb_connection_string: &str,
) -> String {
    format!(
        r#"{base_prompt}

Hard requirements:
1. Use the ADB skill and execute all runtime commands from this container.
2. Run the runtime coordinator script `/runtime/run_runtime_analysis.sh` for collection.
3. Do not skip either runtime plane:
   - Trace plane: `/ebpf/runners/run_trace_experiments.sh`
   - Interaction plane: `/runtime/ui_explorer.py`
4. Record every command you run and any stderr/output errors.

Run context:
- APK path: `/workspace/target.apk`
- Package name: `{package_name}`
- ADB connection string: `{adb_connection_string}`
- runtime coordinator: `/runtime/run_runtime_analysis.sh`
- eBPF probes dir: `/ebpf/probes`
- Report output dir: `/workspace/reports`
- `--trace-phase-seconds` is total across both phases, not per phase.

Required execution sequence:
1. `adb start-server`
2. `adb connect "${{ADB_CONNECTION_STRING}}"`
3. `adb root || true`
4. `adb devices -l`
5. `bash /runtime/run_runtime_analysis.sh --device "${{ADB_CONNECTION_STRING}}" --package '{package_name}' --apk /workspace/target.apk --reports-dir /workspace/reports --trace-phase-seconds 75 --ui-steps 120 --ui-interval-sec 2.0 --ui-monkey-every 0`

Post-processing:
1. Review artifacts generated by the coordinator:
   - `/workspace/reports/runtime-analysis-report.md`
   - `/workspace/reports/runtime-findings.json`
   - `/workspace/reports/runtime-command-log.md`
   - `/workspace/reports/runtime-runner.log`
   - `/workspace/reports/runtime-traces/summary_metrics.csv`
   - `/workspace/reports/runtime-traces/grouped_checks.csv`
2. If any output is missing or incomplete, explain the failure and keep partial evidence.

Deliverables:
1. Ensure these files exist and are updated:
   - `/workspace/reports/runtime-analysis-report.md`
   - `/workspace/reports/runtime-findings.json`
   - `/workspace/reports/runtime-command-log.md`
2. Add a short final summary to `/workspace/reports/runtime-analysis-report.md` if extra context is needed.
"#
    )
}
