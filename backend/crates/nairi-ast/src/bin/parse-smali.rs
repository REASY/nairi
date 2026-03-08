use clap::Parser;
use nairi_ast::ir::EvidenceRef;
use nairi_ast::smali;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory containing smali files
    #[arg(short, long)]
    dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let dir = args.dir;

    if !dir.exists() || !dir.is_dir() {
        eprintln!(
            "Error: Directory '{}' does not exist or is not a directory.",
            dir.display()
        );
        std::process::exit(1);
    }

    println!("Scanning directory: {}", dir.display());

    let smali_files: Vec<(PathBuf, u64)> = WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("smali")
        })
        .map(|e| {
            (
                e.path().to_path_buf(),
                e.metadata().map(|m| m.len()).unwrap_or(0),
            )
        })
        .collect();

    let total_size: u64 = smali_files.iter().map(|(_, size)| size).sum();
    let total_size_mb = total_size as f64 / 1024.0 / 1024.0;

    println!(
        "Found {} smali files ({:.2} MB).",
        smali_files.len(),
        total_size_mb
    );

    let success_count = AtomicUsize::new(0);
    let error_count = AtomicUsize::new(0);
    let syntax_error_count = AtomicUsize::new(0);
    let class_count = AtomicUsize::new(0);
    let method_count = AtomicUsize::new(0);
    let field_count = AtomicUsize::new(0);
    let instr_count = AtomicUsize::new(0);

    let start_time = std::time::Instant::now();

    smali_files.par_iter().for_each(|(path, _size)| {
        let evidence = EvidenceRef {
            run_id: "cli-run".to_string(),
            artifact_id: "cli-artifact".to_string(),
            source: "smali".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        match smali::parse_smali_file(path, evidence) {
            Ok((class_ir, has_errors)) => {
                success_count.fetch_add(1, Ordering::SeqCst);
                if has_errors {
                    syntax_error_count.fetch_add(1, Ordering::SeqCst);
                }
                class_count.fetch_add(1, Ordering::SeqCst);
                method_count.fetch_add(class_ir.methods.len(), Ordering::SeqCst);
                field_count.fetch_add(class_ir.fields.len(), Ordering::SeqCst);

                let instrs: usize = class_ir.methods.iter().map(|m| m.instructions.len()).sum();
                instr_count.fetch_add(instrs, Ordering::SeqCst);
            }
            Err(e) => {
                error_count.fetch_add(1, Ordering::SeqCst);
                eprintln!("Failed to parse {}: {}", path.display(), e);
            }
        }
    });

    let successes = success_count.load(Ordering::SeqCst);
    let errors = error_count.load(Ordering::SeqCst);
    let partial_errors = syntax_error_count.load(Ordering::SeqCst);
    let classes = class_count.load(Ordering::SeqCst);
    let methods = method_count.load(Ordering::SeqCst);
    let fields = field_count.load(Ordering::SeqCst);
    let instructions = instr_count.load(Ordering::SeqCst);
    let elapsed = start_time.elapsed();

    println!("Parsing complete!");
    println!("Successfully parsed files: {}", successes);
    println!("Partial Syntax Errors:   {}", partial_errors);
    println!("Fatal Errors:            {}", errors);
    println!("Classes: {}", classes);
    println!("Methods: {}", methods);
    println!("Fields:  {}", fields);
    println!("Instrs:  {}", instructions);
    println!("Total time: {:?}", elapsed);
    if successes > 0 {
        let avg_time = elapsed / successes as u32;
        println!("Average time per file: {:?}", avg_time);
    }

    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}
