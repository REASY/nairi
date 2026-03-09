use rsmgclient::Connection;

fn main() {
    let mut conn: Connection = unimplemented!();
    let res = conn.execute("MATCH", None).unwrap();
    let rows = conn.fetchall().unwrap();
    for row in rows {
        let _v: &rsmgclient::Value = &row.values[0];
    }
}
