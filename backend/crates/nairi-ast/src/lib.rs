use ir::{ApkIr, ClassIr, EvidenceRef, NativeLibIr};
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

pub mod ghidra;
pub mod ir;
pub mod manifest;
pub mod metrics;
pub mod smali;

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Manifest error: {0}")]
    Manifest(#[from] manifest::ManifestError),
    #[error("Smali parsing error: {0}")]
    Smali(#[from] smali::SmaliError),
    #[error("Ghidra JSON parsing error: {0}")]
    Ghidra(#[from] ghidra::GhidraError),
    #[error("Directory not found or invalid")]
    InvalidDirectory,
}

pub fn parse_directory(
    apk_id: String,
    dir: &Path,
    evidence: EvidenceRef,
) -> Result<ApkIr, ParserError> {
    if !dir.is_dir() {
        return Err(ParserError::InvalidDirectory);
    }

    let manifest_path = dir.join("AndroidManifest.xml");
    let mut manifest_ir = None;
    let mut package_name = None;

    if manifest_path.exists() {
        let manifest = manifest::parse_manifest(&manifest_path, evidence.clone())?;
        package_name = Some(manifest.package.clone());
        manifest_ir = Some(manifest);
    }

    let mut classes: Vec<ClassIr> = Vec::new();
    let smali_dirs = [
        "smali",
        "smali_classes2",
        "smali_classes3",
        "smali_classes4",
    ];

    let mut tracker = StatsTracker::default();
    let start_time = Instant::now();

    for smali_dir_name in smali_dirs {
        let smali_dir = dir.join(smali_dir_name);
        if smali_dir.exists() && smali_dir.is_dir() {
            visit_smali_dir(&smali_dir, &mut classes, evidence.clone(), &mut tracker)?;
        }
    }

    let total_time = start_time.elapsed();
    let avg_time_per_file = if tracker.file_count > 0 {
        total_time.as_secs_f64() / tracker.file_count as f64
    } else {
        0.0
    };

    let mut namespaces = HashSet::new();
    let mut num_interfaces = 0;
    let mut num_methods = 0;

    for c in &classes {
        num_methods += c.methods.len();
        num_interfaces += c.interfaces.len();

        if let Some(idx) = c.descriptor.rfind('/') {
            namespaces.insert(c.descriptor[0..idx].to_string());
        }
    }

    println!("--- AST Parsing Stats ---");
    println!("Total Smali Files: {}", tracker.file_count);
    println!(
        "Total Smali Size: {:.2} MB",
        tracker.total_size as f64 / 1_048_576.0
    );
    println!("Total Parsing Time: {:.2?}", total_time);
    if tracker.file_count > 0 {
        println!(
            "Average Time / File: {:.2} ms",
            (avg_time_per_file * 1000.0)
        );
    }
    println!("Parsed Classes: {}", classes.len());
    println!("Parsed Interfaces: {}", num_interfaces);
    println!("Parsed Namespaces: {}", namespaces.len());
    println!("Parsed Methods: {}", num_methods);
    println!("-------------------------");

    let mut native_libs: Vec<NativeLibIr> = Vec::new();
    let exports_dir = dir.join("ghidra_exports"); // Assuming ghidra outputs json here
    if exports_dir.exists() && exports_dir.is_dir() {
        visit_ghidra_dir(&exports_dir, &mut native_libs, evidence.clone())?;
    }

    Ok(ApkIr {
        apk_id,
        package_name,
        manifest: manifest_ir,
        classes,
        native_libs,
        evidence,
    })
}

#[derive(Default)]
struct StatsTracker {
    total_size: u64,
    file_count: u64,
}

fn visit_smali_dir(
    dir: &Path,
    classes: &mut Vec<ClassIr>,
    evidence: EvidenceRef,
    tracker: &mut StatsTracker,
) -> Result<(), ParserError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_smali_dir(&path, classes, evidence.clone(), tracker)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("smali") {
            tracker.file_count += 1;
            if let Ok(meta) = entry.metadata() {
                tracker.total_size += meta.len();
            }
            // we skip failures on individual files to not panic the entire parsing pipeline
            if let Ok((c, _)) = smali::parse_smali_file(&path, evidence.clone()) {
                classes.push(c);
            }
        }
    }
    Ok(())
}

fn visit_ghidra_dir(
    dir: &Path,
    libs: &mut Vec<NativeLibIr>,
    evidence: EvidenceRef,
) -> Result<(), ParserError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_ghidra_dir(&path, libs, evidence.clone())?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("json")
            && let Ok(l) = ghidra::parse_ghidra_export(&path, evidence.clone())
        {
            libs.push(l);
        }
    }
    Ok(())
}
