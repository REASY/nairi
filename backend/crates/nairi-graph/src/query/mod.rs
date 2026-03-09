use rsmgclient::{Connection, QueryParam, Value};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Database query failed")]
    Database,
}

/// A simple example query to find all components (Activities, Services, etc.) exported by a given APK
pub fn get_exported_components(
    conn: &mut Connection,
    apk_id: &str,
) -> Result<Vec<String>, QueryError> {
    // ENFORCING WHERE MANDATE: All property filtering MUST use a WHERE clause!
    let query = "
        MATCH (a:Apk)-[:HAS_MANIFEST]->(m:Manifest)-[:DECLARES]->(c:Component)
        WHERE a.apk_id = $apk_id AND c.exported = true
        RETURN c.name AS component_name
    ";

    let mut params = HashMap::new();
    params.insert("apk_id".to_string(), QueryParam::String(apk_id.to_string()));

    let _columns = conn
        .execute(query, Some(&params))
        .map_err(|_| QueryError::Database)?;

    let records = conn.fetchall().map_err(|_| QueryError::Database)?;

    let mut extracted = Vec::new();
    for row in records {
        if let Some(Value::String(name)) = row.values.first() {
            extracted.push(name.clone());
        }
    }

    Ok(extracted)
}
