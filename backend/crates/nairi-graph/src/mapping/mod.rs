use nairi_ast::ir::{ApkIr, ClassIr, MethodIr, PermissionIr};
use rsmgclient::{Connection, QueryParam, Value};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum MappingError {
    #[error("Database serialization error: {0}")]
    DatabaseError(String),
}

pub fn insert_apk(conn: &mut Connection, apk: &ApkIr) -> Result<(), MappingError> {
    // Basic Apk Node definition
    let query = "MERGE (a:Apk {apk_id: $apk_id}) \
                 SET a.package_name = $package_name \
                 RETURN a";

    let mut params = HashMap::new();
    params.insert("apk_id".to_string(), QueryParam::String(apk.apk_id.clone()));

    let pkg = apk.package_name.clone().unwrap_or_default();
    params.insert("package_name".to_string(), QueryParam::String(pkg));

    conn.execute(query, Some(&params))
        .map_err(|e| MappingError::DatabaseError(e.to_string()))?;
    conn.fetchall()
        .map_err(|e| MappingError::DatabaseError(e.to_string()))?;

    // Map components, permissions, classes...
    // In MVP, we just prove we can do a couple insertions safely following the WHERE mandate
    Ok(())
}

pub fn insert_class(
    conn: &mut Connection,
    apk_id: &str,
    class: &ClassIr,
) -> Result<(), MappingError> {
    // Enforce WHERE mandate for property filtering: no inline {apk_id: ...} inside MATCH
    let query = "
        MATCH (a:Apk) WHERE a.apk_id = $apk_id
        MERGE (c:Class {descriptor: $descriptor})
        SET c.super_class = $super_class
        MERGE (a)-[:CONTAINS]->(c)
    ";

    let mut params = HashMap::new();
    params.insert("apk_id".to_string(), QueryParam::String(apk_id.to_string()));
    params.insert(
        "descriptor".to_string(),
        QueryParam::String(class.descriptor.clone()),
    );

    let super_c = class.super_class.clone().unwrap_or_default();
    params.insert("super_class".to_string(), QueryParam::String(super_c));

    conn.execute(query, Some(&params))
        .map_err(|e| MappingError::DatabaseError(e.to_string()))?;
    conn.fetchall()
        .map_err(|e| MappingError::DatabaseError(e.to_string()))?;

    for method in &class.methods {
        insert_method(conn, &class.descriptor, method)?;
    }

    Ok(())
}

pub fn insert_method(
    conn: &mut Connection,
    class_descriptor: &str,
    method: &MethodIr,
) -> Result<(), MappingError> {
    let query = "
        MATCH (c:Class) WHERE c.descriptor = $descriptor
        MERGE (m:Method {id: $method_id})
        SET m.name = $name, m.proto = $proto
        MERGE (c)-[:DECLARES]->(m)
    ";

    let mut params = HashMap::new();
    params.insert(
        "descriptor".to_string(),
        QueryParam::String(class_descriptor.to_string()),
    );
    params.insert(
        "method_id".to_string(),
        QueryParam::String(method.id.clone()),
    );
    params.insert("name".to_string(), QueryParam::String(method.name.clone()));
    params.insert(
        "proto".to_string(),
        QueryParam::String(method.proto.clone()),
    );

    conn.execute(query, Some(&params))
        .map_err(|e| MappingError::DatabaseError(e.to_string()))?;
    conn.fetchall()
        .map_err(|e| MappingError::DatabaseError(e.to_string()))?;

    // Instruction parsing/linkage here would go through `InstrIr`
    Ok(())
}
