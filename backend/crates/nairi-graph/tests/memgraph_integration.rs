use nairi_ast::ir::{ApkIr, ClassIr, EvidenceRef, ManifestIr, MethodIr, PermissionIr};
use nairi_graph::ingest::{GraphActorSystem, IngestMessage};
use nairi_graph::mapping;
use nairi_graph::query;
use rsmgclient::{ConnectParams, Connection};
use std::time::Duration;
use testcontainers::{ContainerAsync, GenericImage, ImageExt, runners::AsyncRunner};
use tokio::sync::oneshot;

async fn start_memgraph() -> (ContainerAsync<GenericImage>, ConnectParams) {
    let image = GenericImage::new("memgraph/memgraph-mage", "latest")
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(7687));

    let container = image.start().await.expect("Failed to start Memgraph");

    // Give memgraph a moment to boot
    tokio::time::sleep(Duration::from_secs(2)).await;

    let host = container.get_host().await.unwrap().to_string();
    let port = container.get_host_port_ipv4(7687).await.unwrap();

    let params = ConnectParams {
        host: Some(host),
        port,
        ..Default::default()
    };

    // Wait for the server to be fully ready
    for _ in 0..20 {
        if Connection::connect(&params).is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    (container, params)
}

#[tokio::test]
async fn test_graph_actor_ingestion_and_query() {
    let (_container, params) = start_memgraph().await;

    let sync_params = ConnectParams {
        host: params.host.clone(),
        port: params.port,
        ..Default::default()
    };

    // Start the async GraphActor system
    let actor = GraphActorSystem::spawn_with(params).expect("Failed to spawn GraphActor");

    // 1. Create simulated AST IR
    let evidence = EvidenceRef {
        run_id: "test1".to_string(),
        artifact_id: "test-artifact".to_string(),
        source: "mock".to_string(),
        tool_version: "1.0".to_string(),
    };

    let manifest = ManifestIr {
        package: "com.example.test".to_string(),
        version_code: Some("1".to_string()),
        version_name: Some("1.0".to_string()),
        permissions: vec![PermissionIr {
            name: "android.permission.INTERNET".to_string(),
        }],
        components: vec![],
        evidence: evidence.clone(),
    };

    let apk_ir = ApkIr {
        apk_id: "apk_123".to_string(),
        package_name: Some("com.example.test".to_string()),
        manifest: Some(manifest),
        classes: vec![],
        native_libs: vec![],
        evidence: evidence.clone(),
    };

    let class_ir = ClassIr {
        descriptor: "Lcom/example/test/MainActivity;".to_string(),
        super_class: Some("Landroid/app/Activity;".to_string()),
        interfaces: vec![],
        methods: vec![MethodIr {
            id: "m_1".to_string(),
            name: "onCreate".to_string(),
            proto: "(Landroid/os/Bundle;)V".to_string(),
            access_flags: vec!["public".to_string(), "protected".to_string()],
            instructions: vec![],
            evidence: evidence.clone(),
        }],
        fields: vec![],
        evidence: evidence.clone(),
    };

    // 2. Ingest via Async Actor
    actor
        .insert_apk(apk_ir)
        .await
        .expect("Failed to insert APK");
    actor
        .insert_class("apk_123".to_string(), class_ir)
        .await
        .expect("Failed to insert Class");

    // Give the actor a millisecond to process if it were actually decoupled and batched,
    // though the oneshot channel in GraphActor blocking guarantees it's done.

    // 3. Verify via Query connecting synchronously for raw validation
    let mut sync_conn = Connection::connect(&sync_params).unwrap();

    // Check Apk node
    let _cols = sync_conn
        .execute(
            "MATCH (a:Apk) WHERE a.apk_id = 'apk_123' RETURN a.package_name",
            None,
        )
        .unwrap();
    let rows = sync_conn.fetchall().unwrap();
    assert_eq!(rows.len(), 1);

    let _ = sync_conn
        .execute("MATCH (c:Class) RETURN count(c)", None)
        .unwrap();
    let class_count = sync_conn.fetchall().unwrap();
    println!("Class count: {:?}", class_count[0].values[0]);

    let _ = sync_conn
        .execute("MATCH (m:Method) RETURN count(m)", None)
        .unwrap();
    let method_count = sync_conn.fetchall().unwrap();
    println!("Method count: {:?}", method_count[0].values[0]);

    // Check Class node relationships
    let query = "
        MATCH (a:Apk)-[:CONTAINS]->(c:Class)-[:DECLARES]->(m:Method)
        WHERE a.apk_id = 'apk_123' AND c.descriptor = 'Lcom/example/test/MainActivity;' AND m.name = 'onCreate'
        RETURN m.proto
    ";
    let _cols2 = sync_conn.execute(query, None).unwrap();
    let method_rows = sync_conn.fetchall().unwrap();
    assert_eq!(method_rows.len(), 1);

    // Validate the cypher extracted proto
    if let Some(rsmgclient::Value::String(proto)) = method_rows[0].values.get(0) {
        assert_eq!(proto, "(Landroid/os/Bundle;)V");
    } else {
        panic!("Missing method proto");
    }
}
