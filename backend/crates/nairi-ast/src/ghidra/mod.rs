use serde::Deserialize;
use std::path::Path;

use crate::ir::{EvidenceRef, NativeFunctionIr, NativeImportIr, NativeLibIr};

#[derive(Debug, thiserror::Error)]
pub enum GhidraError {
    #[error("Could not read Ghidra export file: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Represents the raw JSON output produced by the Ghidra headless script.
#[derive(Debug, Deserialize)]
pub struct GhidraExport {
    pub path: String,
    pub abi: String,
    pub sha256: String,
    pub exports: Vec<GhidraFunction>,
    pub imports: Vec<GhidraImport>,
    // we can ignore other fields like "strings" for the IR mapping if not specified in spec explicitly
}

#[derive(Debug, Deserialize)]
pub struct GhidraFunction {
    pub name: String,
    pub address: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GhidraImport {
    pub symbol: String,
}

pub fn parse_ghidra_export(
    json_path: &Path,
    evidence: EvidenceRef,
) -> Result<NativeLibIr, GhidraError> {
    let content = std::fs::read_to_string(json_path)?;
    let export: GhidraExport = serde_json::from_str(&content)?;

    let exports = export
        .exports
        .into_iter()
        .map(|f| NativeFunctionIr {
            id: uuid::Uuid::new_v4().to_string(), // Ephemeral ID for the graph
            name: f.name,
            address: f.address,
        })
        .collect();

    let imports = export
        .imports
        .into_iter()
        .map(|i| NativeImportIr {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: i.symbol,
        })
        .collect();

    Ok(NativeLibIr {
        id: uuid::Uuid::new_v4().to_string(),
        abi: export.abi,
        path: export.path,
        sha256: export.sha256,
        exports,
        imports,
        evidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_ghidra_export() {
        let json = r#"{
            "path": "lib/arm64-v8a/libnative.so",
            "abi": "arm64-v8a",
            "sha256": "0123456789abcdef",
            "exports": [
                {"name": "Java_com_example_app_MainActivity_stringFromJNI", "address": 4096}
            ],
            "imports": [
                {"symbol": "system"},
                {"symbol": "SSL_set_custom_verify"}
            ]
        }"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let export_path = temp_dir.path().join("export.json");
        let mut file = std::fs::File::create(&export_path).unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let evidence = EvidenceRef {
            run_id: "r1".to_string(),
            artifact_id: "a1".to_string(),
            source: "ghidra".to_string(),
            tool_version: "2.0".to_string(),
        };

        let result = parse_ghidra_export(&export_path, evidence).unwrap();
        assert_eq!(result.abi, "arm64-v8a");
        assert_eq!(result.path, "lib/arm64-v8a/libnative.so");
        assert_eq!(result.sha256, "0123456789abcdef");
        assert_eq!(result.exports.len(), 1);
        assert_eq!(
            result.exports[0].name,
            "Java_com_example_app_MainActivity_stringFromJNI"
        );
        assert_eq!(result.exports[0].address, Some(4096));
        assert_eq!(result.imports.len(), 2);
        assert_eq!(result.imports[0].symbol, "system");
        assert_eq!(result.imports[1].symbol, "SSL_set_custom_verify");
    }
}
