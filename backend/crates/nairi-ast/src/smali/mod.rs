use std::path::Path;
use tree_sitter::{Node, Parser};

use crate::ir::{ClassIr, EvidenceRef, FieldIr, InstrIr, MethodIr};

#[derive(Debug, thiserror::Error)]
pub enum SmaliError {
    #[error("Failed to read smali file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse smali with tree-sitter")]
    ParseError,
    #[error("Tree-sitter encountered syntax errors")]
    SyntaxError,
}

pub fn parse_smali_file(path: &Path, evidence: EvidenceRef) -> Result<(ClassIr, bool), SmaliError> {
    let code = std::fs::read_to_string(path)?;
    parse_smali_content(&code, evidence)
}

pub fn parse_smali_content(
    code: &str,
    evidence: EvidenceRef,
) -> Result<(ClassIr, bool), SmaliError> {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_smali::language())
        .expect("Failed to load smali language");

    let tree = parser.parse(code, None).ok_or(SmaliError::ParseError)?;
    let root = tree.root_node();
    let has_syntax_errors = root.has_error();

    if root.kind() == "class_definition" {
        return Ok((extract_class_def(root, code, evidence), has_syntax_errors));
    }

    // A smali file might be wrapped in source_file
    for child in root.children(&mut root.walk()) {
        if child.kind() == "class_definition" {
            return Ok((extract_class_def(child, code, evidence), has_syntax_errors));
        }
    }

    Err(SmaliError::ParseError)
}

fn extract_class_def(node: Node, source: &str, evidence: EvidenceRef) -> ClassIr {
    let mut descriptor = String::new();
    let mut super_class = None;
    let mut interfaces = Vec::new();
    let mut fields = Vec::new();
    let mut methods = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "class_directive" => {
                descriptor =
                    extract_text_by_kind(child, "class_identifier", source).unwrap_or_default();
            }
            "super_directive" => {
                super_class = extract_text_by_kind(child, "class_identifier", source);
            }
            "implements_directive" => {
                if let Some(iface) = extract_text_by_kind(child, "class_identifier", source) {
                    interfaces.push(iface);
                }
            }
            "field_definition" => {
                fields.push(extract_field_def(child, source));
            }
            "method_definition" => {
                methods.push(extract_method_def(child, source, evidence.clone()));
            }
            _ => {}
        }
    }

    ClassIr {
        descriptor,
        super_class,
        interfaces,
        methods,
        fields,
        evidence,
    }
}

fn extract_field_def(node: Node, source: &str) -> FieldIr {
    let name = extract_text_by_kind(node, "field_identifier", source).unwrap_or_default();
    let access_flags = extract_access_modifiers(node, source);
    let field_type = extract_text_by_kind(node, "field_type", source).unwrap_or_default();

    // Simplistic initializer extraction
    let initial_value = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "number" || c.kind() == "string_value")
        .map(|c| c.utf8_text(source.as_bytes()).unwrap().to_string());

    FieldIr {
        name,
        field_type,
        access_flags,
        initial_value,
    }
}

fn extract_method_def(node: Node, source: &str, evidence: EvidenceRef) -> MethodIr {
    let mut name = String::new();
    let mut proto = String::new();
    let access_flags = extract_access_modifiers(node, source);
    let mut instructions = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "method_signature" {
            name = extract_text_by_kind(child, "method_identifier", source).unwrap_or_default();
            // Just take the whole signature text as proto for now
            proto = child.utf8_text(source.as_bytes()).unwrap().to_string();
        } else if child.kind() == "expression"
            && let Some(instr) = extract_instruction(child, source)
        {
            instructions.push(instr);
        }
    }

    MethodIr {
        id: uuid::Uuid::new_v4().to_string(), // Ephemeral ID for graph linking
        name,
        proto,
        access_flags,
        instructions,
        evidence,
    }
}

fn extract_instruction(node: Node, source: &str) -> Option<InstrIr> {
    let opcode_node = node.child(0)?;
    let opcode = opcode_node
        .utf8_text(source.as_bytes())
        .unwrap()
        .to_string();

    if opcode.starts_with("invoke-") {
        // Find method_signature inside body
        let mut target = String::new();
        if let Some(body) = get_child_by_kind(node, "body")
            && let Some(class_id) = get_child_by_kind(body, "full_method_signature")
        {
            target = class_id.utf8_text(source.as_bytes()).unwrap().to_string();
        }
        Some(InstrIr::Invoke { target })
    } else if opcode == "const-string" {
        let value = extract_text_by_kind(node, "string_value", source).unwrap_or_default();
        Some(InstrIr::ConstString { value })
    } else if opcode.starts_with("sget") || opcode.starts_with("iget") {
        Some(InstrIr::FieldRead {
            field: "TODO".to_string(),
        })
    } else if opcode.starts_with("sput") || opcode.starts_with("iput") {
        Some(InstrIr::FieldWrite {
            field: "TODO".to_string(),
        })
    } else {
        Some(InstrIr::Other { opcode })
    }
}

fn extract_access_modifiers(node: Node, source: &str) -> Vec<String> {
    let mut acc = Vec::new();

    let mut extract_from_node = |n: Node| {
        let text = n.utf8_text(source.as_bytes()).unwrap_or_default();
        for word in text.split_whitespace() {
            if !word.is_empty() {
                acc.push(word.to_string());
            }
        }
    };

    if let Some(am_node) = get_child_by_kind(node, "access_modifiers") {
        for child in am_node.children(&mut am_node.walk()) {
            if child.kind() == "access_modifier" {
                extract_from_node(child);
            }
        }
    }
    // single modifier without container
    if node.kind() == "method_definition" {
        for child in node.children(&mut node.walk()) {
            if child.kind() == "access_modifier" {
                extract_from_node(child);
            }
        }
    }
    acc
}

fn extract_text_by_kind(node: Node, kind: &str, source: &str) -> Option<String> {
    get_child_by_kind(node, kind).map(|n| n.utf8_text(source.as_bytes()).unwrap().to_string())
}

fn get_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|&child| child.kind() == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_smali_content() {
        let code = r#"
.class public Lcom/example/MyClass;
.super Ljava/lang/Object;
.implements Ljava/lang/Runnable;

.field public static final MY_CONST:I = 0x1

.method public constructor <init>()V
    .registers 1
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V
    return-void
.end method
        "#;

        let evidence = EvidenceRef {
            run_id: "r1".to_string(),
            artifact_id: "a1".to_string(),
            source: "smali".to_string(),
            tool_version: "1.0".to_string(),
        };

        let (result, has_errors) = parse_smali_content(code, evidence).unwrap();
        assert!(!has_errors);
        assert_eq!(result.descriptor, "Lcom/example/MyClass;");
        assert_eq!(result.super_class, Some("Ljava/lang/Object;".to_string()));
        assert_eq!(result.interfaces, vec!["Ljava/lang/Runnable;"]);
        assert_eq!(result.fields.len(), 1);
        assert_eq!(result.fields[0].name, "MY_CONST");
        assert_eq!(
            result.fields[0].access_flags,
            vec!["public", "static", "final"]
        );

        assert_eq!(result.methods.len(), 1);
        let m = &result.methods[0];
        assert_eq!(m.name, "<init>");
        assert_eq!(m.access_flags, vec!["public"]);
        assert_eq!(m.instructions.len(), 2);
    }
}
