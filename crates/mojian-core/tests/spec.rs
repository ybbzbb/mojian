use std::path::{Path, PathBuf};

use mojian_core::{
    authoritative_hash, authoritative_version, deploy_spec, embedded_spec, ensure_master,
    sync_if_drifted, tree_hash,
};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mojian-spec-test-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_file(root: &Path, rel: &str, content: &[u8]) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, content).unwrap();
}

fn bootstrapped_master(tag: &str) -> (PathBuf, PathBuf) {
    let data_dir = unique_dir(tag);
    let master = data_dir.join("spec");
    ensure_master(embedded_spec(), &master).expect("bootstrap master");
    (data_dir, master)
}

#[test]
fn bootstrap_writes_master_tree_and_version() {
    let (data_dir, master) = bootstrapped_master("bootstrap");

    assert!(master.join("spec.toml").is_file());
    assert!(master.join("CLAUDE.md").is_file());
    assert!(master.join(".claude/agents/.gitkeep").is_file());
    assert!(master.join(".claude/skills/.gitkeep").is_file());
    assert!(master.join("prompts/sop-1-style/README.md").is_file());
    assert!(master.join("prompts/sop-2-bible/README.md").is_file());
    assert!(master.join("prompts/sop-3-writing/README.md").is_file());

    let version = authoritative_version(&master).expect("read version");
    assert_eq!(version, "0.0.1-skeleton");

    std::fs::remove_dir_all(&data_dir).ok();
}

#[test]
fn deploy_places_targets_without_spec_toml_and_hash_matches_authoritative() {
    let (data_dir, master) = bootstrapped_master("deploy");
    let project = unique_dir("deploy-project");

    let (version, hash) = deploy_spec(&master, &project).expect("deploy");
    assert_eq!(version, "0.0.1-skeleton");

    assert!(project.join(".claude/agents/.gitkeep").is_file());
    assert!(project.join(".claude/skills/.gitkeep").is_file());
    assert!(project.join("CLAUDE.md").is_file());
    assert!(project.join("prompts/sop-1-style/README.md").is_file());
    assert!(project.join("prompts/sop-2-bible/README.md").is_file());
    assert!(project.join("prompts/sop-3-writing/README.md").is_file());
    assert!(
        !project.join("spec.toml").exists(),
        "spec.toml 不属部署载荷，不应出现在项目内"
    );

    let expected = authoritative_hash(&master).expect("authoritative hash");
    assert_eq!(hash, expected, "部署返回 hash 应等于权威载荷 hash");

    std::fs::remove_dir_all(&data_dir).ok();
    std::fs::remove_dir_all(&project).ok();
}

#[test]
fn tree_hash_is_order_independent_and_content_sensitive() {
    let dir_a = unique_dir("hash-a");
    let dir_b = unique_dir("hash-b");

    // 相同内容、不同创建顺序 → 同 hash。
    write_file(&dir_a, "a.txt", b"alpha");
    write_file(&dir_a, "sub/b.txt", b"beta");

    write_file(&dir_b, "sub/b.txt", b"beta");
    write_file(&dir_b, "a.txt", b"alpha");

    let hash_a = tree_hash(&dir_a).expect("hash a");
    let hash_b = tree_hash(&dir_b).expect("hash b");
    assert_eq!(hash_a, hash_b, "顺序无关：同内容不同顺序应同 hash");

    // 改一个文件内容 → 不同 hash。
    let dir_c = unique_dir("hash-c");
    write_file(&dir_c, "a.txt", b"alpha-CHANGED");
    write_file(&dir_c, "sub/b.txt", b"beta");
    let hash_c = tree_hash(&dir_c).expect("hash c");
    assert_ne!(hash_a, hash_c, "内容敏感：改内容应改 hash");

    std::fs::remove_dir_all(&dir_a).ok();
    std::fs::remove_dir_all(&dir_b).ok();
    std::fs::remove_dir_all(&dir_c).ok();
}

#[test]
fn sync_if_drifted_no_drift_does_not_overwrite() {
    let (data_dir, master) = bootstrapped_master("nodrift");
    let project = unique_dir("nodrift-project");

    let (_version, hash) = deploy_spec(&master, &project).expect("deploy");
    let (overwritten, current) =
        sync_if_drifted(&project, &hash, &master).expect("sync no drift");

    assert!(!overwritten, "未漂移时不应覆盖");
    assert_eq!(current, hash);

    std::fs::remove_dir_all(&data_dir).ok();
    std::fs::remove_dir_all(&project).ok();
}

#[test]
fn sync_if_drifted_restores_tampered_file() {
    let (data_dir, master) = bootstrapped_master("tamper");
    let project = unique_dir("tamper-project");

    let (_version, hash) = deploy_spec(&master, &project).expect("deploy");
    let claude = project.join("CLAUDE.md");
    let original = std::fs::read(&claude).unwrap();

    std::fs::write(&claude, b"tampered content").unwrap();
    let (overwritten, new_hash) =
        sync_if_drifted(&project, &hash, &master).expect("sync after tamper");

    assert!(overwritten, "篡改后应触发覆盖");
    assert_eq!(new_hash, hash, "覆盖后 hash 应还原为权威值");
    assert_eq!(
        std::fs::read(&claude).unwrap(),
        original,
        "被篡改文件内容应被还原"
    );

    std::fs::remove_dir_all(&data_dir).ok();
    std::fs::remove_dir_all(&project).ok();
}

#[test]
fn sync_if_drifted_restores_deleted_file() {
    let (data_dir, master) = bootstrapped_master("deleted");
    let project = unique_dir("deleted-project");

    let (_version, hash) = deploy_spec(&master, &project).expect("deploy");
    let readme = project.join("prompts/sop-1-style/README.md");
    std::fs::remove_file(&readme).unwrap();

    let (overwritten, new_hash) =
        sync_if_drifted(&project, &hash, &master).expect("sync after delete");

    assert!(overwritten, "删除部署文件后应触发覆盖");
    assert_eq!(new_hash, hash);
    assert!(readme.is_file(), "被删文件应被重部署还原");

    std::fs::remove_dir_all(&data_dir).ok();
    std::fs::remove_dir_all(&project).ok();
}
