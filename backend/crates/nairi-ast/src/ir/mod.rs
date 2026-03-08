use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub run_id: String,
    pub artifact_id: String,
    pub source: String, // manifest|smali|ghidra
    pub tool_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApkIr {
    pub apk_id: String,
    pub package_name: Option<String>,
    pub manifest: Option<ManifestIr>,
    pub classes: Vec<ClassIr>,
    pub native_libs: Vec<NativeLibIr>,
    pub evidence: EvidenceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestIr {
    pub package: String,
    pub version_code: Option<String>,
    pub version_name: Option<String>,
    pub permissions: Vec<PermissionIr>,
    pub components: Vec<ComponentIr>,
    pub evidence: EvidenceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionIr {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    Activity,
    Service,
    Receiver,
    Provider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentIr {
    pub name: String,
    pub component_type: ComponentType,
    pub exported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassIr {
    pub descriptor: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub methods: Vec<MethodIr>,
    pub fields: Vec<FieldIr>,
    pub evidence: EvidenceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldIr {
    pub name: String,
    pub field_type: String,
    pub access_flags: Vec<String>,
    /// Optional constant initializer if the field has one
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodIr {
    pub id: String,
    pub name: String,
    pub proto: String,
    pub access_flags: Vec<String>,
    pub instructions: Vec<InstrIr>,
    pub evidence: EvidenceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstrIr {
    Invoke { target: String },
    LoadLibrary { lib_name: String },
    ConstString { value: String },
    FieldRead { field: String },
    FieldWrite { field: String },
    Other { opcode: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeLibIr {
    pub id: String,
    pub abi: String,
    pub path: String,
    pub sha256: String,
    pub exports: Vec<NativeFunctionIr>,
    pub imports: Vec<NativeImportIr>,
    pub evidence: EvidenceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeFunctionIr {
    pub id: String,
    pub name: String,
    pub address: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeImportIr {
    pub id: String,
    pub symbol: String,
}
