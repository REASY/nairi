use std::path::Path;

use crate::ir::{ComponentIr, ComponentType, EvidenceRef, ManifestIr, PermissionIr};

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Could not read manifest file: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML parsing error: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("Missing package attribute in manifest tag")]
    MissingPackage,
}

pub fn parse_manifest(
    manifest_path: &Path,
    evidence: EvidenceRef,
) -> Result<ManifestIr, ManifestError> {
    let xml_str = std::fs::read_to_string(manifest_path)?;
    let doc = roxmltree::Document::parse(&xml_str)?;

    let manifest_node = doc
        .root()
        .children()
        .find(|n| n.has_tag_name("manifest"))
        .ok_or(ManifestError::MissingPackage)?;

    let package = manifest_node
        .attribute("package")
        .ok_or(ManifestError::MissingPackage)?
        .to_string();

    let version_code = manifest_node
        .attribute(("http://schemas.android.com/apk/res/android", "versionCode"))
        .map(|s| s.to_string());

    let version_name = manifest_node
        .attribute(("http://schemas.android.com/apk/res/android", "versionName"))
        .map(|s| s.to_string());

    let mut permissions = Vec::new();
    let mut components = Vec::new();

    for child in manifest_node.children() {
        match child.tag_name().name() {
            "uses-permission" => {
                if let Some(name) =
                    child.attribute(("http://schemas.android.com/apk/res/android", "name"))
                {
                    permissions.push(PermissionIr {
                        name: name.to_string(),
                    });
                }
            }
            "application" => {
                for app_child in child.children() {
                    let c_type = match app_child.tag_name().name() {
                        "activity" | "activity-alias" => Some(ComponentType::Activity),
                        "service" => Some(ComponentType::Service),
                        "receiver" => Some(ComponentType::Receiver),
                        "provider" => Some(ComponentType::Provider),
                        _ => None,
                    };

                    if let Some(component_type) = c_type
                        && let Some(name) = app_child
                        .attribute(("http://schemas.android.com/apk/res/android", "name"))
                    {
                        // Determine if component is exported.
                        // If `android:exported` is explicitly set, use it.
                        // Otherwise, if it has intent-filters, it's implicitly true (before Android 12) or false (no intent filter).
                        // A proper heuristic checks for intent-filters.
                        let exported_attr = app_child.attribute((
                            "http://schemas.android.com/apk/res/android",
                            "exported",
                        ));

                        let exported = match exported_attr {
                            Some("true") => true,
                            Some("false") => false,
                            None => {
                                // Check if it has an intent-filter
                                app_child
                                    .children()
                                    .any(|n| n.has_tag_name("intent-filter"))
                            }
                            _ => false,
                        };

                        let mut full_name = name.to_string();
                        if full_name.starts_with('.') {
                            full_name = format!("{}{}", package, name);
                        } else if !full_name.contains('.') {
                            full_name = format!("{}.{}", package, name);
                        }

                        components.push(ComponentIr {
                            name: full_name,
                            component_type,
                            exported,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    Ok(ManifestIr {
        package,
        version_code,
        version_name,
        permissions,
        components,
        evidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_manifest() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.example.app"
    android:versionCode="1"
    android:versionName="1.0">
    <uses-permission android:name="android.permission.INTERNET" />
    <application>
        <activity android:name=".MainActivity" android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
        <service android:name="com.example.app.MyService" />
        <receiver android:name=".MyReceiver">
            <intent-filter>
                <action android:name="android.intent.action.BOOT_COMPLETED" />
            </intent-filter>
        </receiver>
    </application>
</manifest>"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("AndroidManifest.xml");
        let mut file = std::fs::File::create(&manifest_path).unwrap();
        file.write_all(xml.as_bytes()).unwrap();

        let evidence = EvidenceRef {
            run_id: "r1".to_string(),
            artifact_id: "a1".to_string(),
            source: "manifest".to_string(),
            tool_version: "1.0".to_string(),
        };

        let result = parse_manifest(&manifest_path, evidence).unwrap();
        assert_eq!(result.package, "com.example.app");
        assert_eq!(result.version_code, Some("1".to_string()));
        assert_eq!(result.permissions.len(), 1);
        assert_eq!(result.permissions[0].name, "android.permission.INTERNET");

        assert_eq!(result.components.len(), 3);

        let main = result
            .components
            .iter()
            .find(|c| c.name == "com.example.app.MainActivity")
            .unwrap();
        assert_eq!(main.component_type, ComponentType::Activity);
        assert_eq!(main.exported, true);

        let srv = result
            .components
            .iter()
            .find(|c| c.name == "com.example.app.MyService")
            .unwrap();
        assert_eq!(srv.component_type, ComponentType::Service);
        assert_eq!(srv.exported, false);

        let rec = result
            .components
            .iter()
            .find(|c| c.name == "com.example.app.MyReceiver")
            .unwrap();
        assert_eq!(rec.component_type, ComponentType::Receiver);
        // Implicitly true because it has an intent-filter
        assert_eq!(rec.exported, true);
    }
}
