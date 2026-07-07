use std::path::PathBuf;

use mojian_core::open_central_db;

/// storage.md「五」的 12 个具名表，全部必须在建库后存在于 sqlite_master。
const EXPECTED_TABLES: [&str; 12] = [
    "project",
    "project_state",
    "reference_book",
    "volume",
    "batch",
    "chapter",
    "artifact_ref",
    "bible_version",
    "void_record",
    "stat",
    "config",
    "schema_meta",
];

fn unique_db_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mojian-db-test-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir.join("central.db")
}

fn table_names(conn: &rusqlite::Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .unwrap();
    let names = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    names
}

fn schema_version(conn: &rusqlite::Connection) -> i64 {
    conn.query_row("SELECT schema_version FROM schema_meta", [], |row| row.get(0))
        .unwrap()
}

#[test]
fn db_fresh_creates_all_twelve_tables_and_stamps_version() {
    let path = unique_db_path("fresh");
    let conn = open_central_db(&path).expect("open_central_db builds fresh db");

    let names = table_names(&conn);
    for table in EXPECTED_TABLES {
        assert!(
            names.iter().any(|n| n == table),
            "缺少表 {table}，实际存在：{names:?}"
        );
    }
    // 只应有 12 张具名业务表（sqlite_autoindex_* 不出现在 type='table' 查询里）。
    let business_tables: Vec<&String> = names
        .iter()
        .filter(|n| !n.starts_with("sqlite_"))
        .collect();
    assert_eq!(
        business_tables.len(),
        12,
        "期望恰好 12 张表，实际：{business_tables:?}"
    );

    assert_eq!(schema_version(&conn), 1);

    drop(conn);
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn db_reopening_same_path_is_idempotent() {
    let path = unique_db_path("idempotent");

    let first = open_central_db(&path).expect("first open");
    assert_eq!(schema_version(&first), 1);
    let first_tables = table_names(&first);
    drop(first);

    // 二次打开同一路径：不重复建表、不报错、版本仍为 1。
    let second = open_central_db(&path).expect("second open is idempotent");
    assert_eq!(schema_version(&second), 1);
    assert_eq!(table_names(&second), first_tables);

    // schema_meta 仍只有一行版本记录。
    let row_count: i64 = second
        .query_row("SELECT count(*) FROM schema_meta", [], |row| row.get(0))
        .unwrap();
    assert_eq!(row_count, 1);

    drop(second);
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn db_foreign_keys_pragma_is_enabled() {
    let path = unique_db_path("fk");
    let conn = open_central_db(&path).expect("open");
    let fk_on: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(fk_on, 1, "open_central_db 应启用 foreign_keys");

    drop(conn);
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}
