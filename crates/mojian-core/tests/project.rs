use std::path::PathBuf;

use mojian_core::{
    load_project_state, open_central_db, read_manifest, register_project, write_manifest,
    CoreError, ProjectManifest, SopPhase,
};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mojian-project-test-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn project_register_then_load_returns_style_sampling() {
    let dir = unique_dir("register");
    let db_path = dir.join("central.db");
    let mut conn = open_central_db(&db_path).expect("open central db");

    let project_dir = dir.join("proj");
    std::fs::create_dir_all(&project_dir).unwrap();

    let project_id =
        register_project(&mut conn, "proj", &project_dir).expect("register project");
    assert!(!project_id.is_empty(), "project_id 应为非空 UUID");

    let phase = load_project_state(&conn, &project_id).expect("load project state");
    assert_eq!(phase, SopPhase::StyleSampling);

    // 存入的 path 应为绝对路径。
    let stored_path: String = conn
        .query_row(
            "SELECT path FROM project WHERE project_id = ?1",
            [&project_id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(PathBuf::from(&stored_path).is_absolute(), "path 应绝对：{stored_path}");

    drop(conn);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn project_load_unknown_is_err() {
    let dir = unique_dir("unknown");
    let db_path = dir.join("central.db");
    let conn = open_central_db(&db_path).expect("open central db");

    let err = load_project_state(&conn, "no-such-id").unwrap_err();
    assert!(matches!(err, CoreError::Db(_)), "无此项目应返回 Db 错误：{err:?}");

    drop(conn);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn project_manifest_write_read_roundtrip() {
    let dir = unique_dir("manifest");

    let manifest = ProjectManifest {
        project_id: "11111111-2222-3333-4444-555555555555".to_string(),
        spec_version: "0.0.1-skeleton".to_string(),
    };
    write_manifest(&dir, &manifest).expect("write manifest");

    let back = read_manifest(&dir).expect("read manifest");
    assert_eq!(back, manifest);
    assert_eq!(back.project_id, manifest.project_id);
    assert_eq!(back.spec_version, manifest.spec_version);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn project_read_missing_manifest_is_err() {
    let dir = unique_dir("missing-manifest");
    let err = read_manifest(&dir).unwrap_err();
    assert!(matches!(err, CoreError::Io { .. }), "缺文件应返回 Io 错误：{err:?}");
    std::fs::remove_dir_all(&dir).ok();
}
