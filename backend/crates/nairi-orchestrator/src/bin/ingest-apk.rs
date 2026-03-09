use clap::Parser;
use rsmgclient::ConnectParams;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Direct deterministic AST to Memgraph ingestion", long_about = None
)]
struct Args {
    #[arg(
        short,
        long,
        help = "Directory containing the decompiled APK (smali/manifest/etc.)"
    )]
    dir: PathBuf,

    /// Number of classes to insert per transaction batch
    #[arg(short, long, default_value_t = 5000)]
    chunk_size: usize,

    #[arg(long, default_value = "127.0.0.1", help = "Memgraph host address")]
    host: String,

    #[arg(long, default_value_t = 7687, help = "Memgraph port")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    let params = ConnectParams {
        host: Some(args.host),
        port: args.port,
        ..Default::default()
    };

    println!("Starting ingestion pipeline on: {:?}", args.dir);
    println!(
        "Targeting Memgraph at {}:{}",
        params.host.as_ref().unwrap(),
        params.port
    );

    nairi_orchestrator::pipeline::run_ast_pipeline_on_dir(&args.dir, params, args.chunk_size)
        .await?;

    println!("✅ Ingestion completed successfully.");
    Ok(())
}
