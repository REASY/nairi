use nairi_ast::ir::EvidenceRef;
use nairi_graph::ingest::GraphActorSystem;
use rsmgclient::ConnectParams;
use std::path::PathBuf;

pub async fn run_ast_pipeline_on_dir(
    apk_dir: &PathBuf,
    conn_params: ConnectParams,
    chunk_size: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Spawning GraphActorSystem...");
    let actor = GraphActorSystem::spawn_with(conn_params)?;

    println!("Parsing directory: {:?}", apk_dir);
    let evidence = EvidenceRef {
        run_id: "test-run".to_string(),
        artifact_id: "test-artifact".to_string(),
        source: "smali".to_string(),
        tool_version: "1.0".to_string(),
    };
    let apk_ir = nairi_ast::parse_directory("test_apk".to_string(), apk_dir, evidence)?;

    println!("Parsed APK IR with {} classes", apk_ir.classes.len());

    println!("Inserting APK node...");
    actor.insert_apk(apk_ir.clone()).await?;

    println!("Inserting {} classes...", apk_ir.classes.len());
    for chunk in apk_ir.classes.chunks(chunk_size) {
        let s = std::time::Instant::now();
        actor
            .insert_classes(apk_ir.apk_id.clone(), chunk.to_vec())
            .await?;
        println!(
            "Inserted chunk of {} classes in {:?}",
            chunk.len(),
            s.elapsed()
        );
    }

    println!("Ingestion complete!");
    Ok(())
}
