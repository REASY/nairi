use crate::db::{Memgraph, MemgraphError, QuerySpec};
use nairi_ast::ir::{ApkIr, ClassIr};
use rsmgclient::{ConnectParams, QueryParam};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum MappingError {
    #[error("Database serialization error: {0}")]
    DatabaseError(String),
}

impl From<MemgraphError> for MappingError {
    fn from(err: MemgraphError) -> Self {
        MappingError::DatabaseError(err.to_string())
    }
}

pub fn init_indices(params: &ConnectParams) -> Result<(), MappingError> {
    let index_params = ConnectParams {
        host: params.host.clone(),
        port: params.port,
        autocommit: true,
        lazy: params.lazy,
        ..Default::default()
    };
    let mut db = Memgraph::try_new(&index_params)?;

    let queries = [
        "CREATE INDEX ON :Apk(apk_id);",
        "CREATE INDEX ON :Class(descriptor);",
        "CREATE INDEX ON :Method(id);",
    ];

    for query in queries {
        // Run index queries; on error (e.g. index exists), Memgraph DB wrapper
        // will safely tear down and reconnect exactly as we want.
        let spec = QuerySpec::new(query.to_string());
        if let Err(e) = db.execute_query_spec(&spec) {
            eprintln!("Index creation returned (likely already exists): {}", e);
        }
    }

    Ok(())
}

pub fn insert_apk(db: &mut Memgraph, apk: &ApkIr) -> Result<(), MappingError> {
    let mut params = HashMap::new();
    params.insert("apk_id".to_string(), QueryParam::String(apk.apk_id.clone()));
    params.insert(
        "package_name".to_string(),
        QueryParam::String(apk.package_name.clone().unwrap_or_default()),
    );
    let version_code_val = apk
        .manifest
        .as_ref()
        .and_then(|m| m.version_code.clone())
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);
    params.insert(
        "version_code".to_string(),
        QueryParam::Int(version_code_val),
    );
    let version_name_val = apk
        .manifest
        .as_ref()
        .and_then(|m| m.version_name.clone())
        .unwrap_or_default();
    params.insert(
        "version_name".to_string(),
        QueryParam::String(version_name_val),
    );

    let query = "
        MERGE (a:Apk {apk_id: $apk_id})
        SET a.package_name = $package_name,
            a.version_code = $version_code,
            a.version_name = $version_name
    ";

    let spec = QuerySpec::with_params(query.to_string(), params);
    db.execute_query_spec(&spec)?;

    Ok(())
}

pub fn insert_class(db: &mut Memgraph, apk_id: &str, class: &ClassIr) -> Result<(), MappingError> {
    insert_classes(db, apk_id, &[class.clone()])
}

pub fn insert_classes(
    db: &mut Memgraph,
    apk_id: &str,
    classes: &[ClassIr],
) -> Result<(), MappingError> {
    if classes.is_empty() {
        return Ok(());
    }

    let mut class_params = Vec::new();
    for class in classes {
        let mut map = HashMap::new();
        map.insert(
            "descriptor".to_string(),
            QueryParam::String(class.descriptor.clone()),
        );

        let mut method_params = Vec::new();
        for method in &class.methods {
            let mut m_map = HashMap::new();
            m_map.insert("id".to_string(), QueryParam::String(method.id.clone()));
            m_map.insert("name".to_string(), QueryParam::String(method.name.clone()));
            m_map.insert(
                "signature".to_string(),
                QueryParam::String(method.proto.clone()),
            );
            method_params.push(QueryParam::Map(m_map));
        }
        map.insert("methods".to_string(), QueryParam::List(method_params));

        class_params.push(QueryParam::Map(map));
    }

    let mut params = HashMap::new();
    params.insert("apk_id".to_string(), QueryParam::String(apk_id.to_string()));
    params.insert("classes".to_string(), QueryParam::List(class_params));

    let query = "
        MATCH (a:Apk) WHERE a.apk_id = $apk_id
        UNWIND $classes AS class
        MERGE (c:Class {descriptor: class.descriptor})
        MERGE (a)-[:CONTAINS_CLASS]->(c)
        FOREACH (method IN coalesce(class.methods, []) |
            MERGE (m:Method {id: method.id})
            SET m.name = method.name, m.signature = method.signature
            MERGE (c)-[:DECLARES_METHOD]->(m)
        )
    ";

    let spec = QuerySpec::with_params(query.to_string(), params);
    db.execute_query_spec(&spec)?;

    Ok(())
}
