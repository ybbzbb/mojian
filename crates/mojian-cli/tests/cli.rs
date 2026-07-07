//! `mojian` 端到端集成测试：以 `MOJIAN_HOME` 指向隔离临时目录，驱动真实二进制。
//!
//! 覆盖 new→status 正常路径、「非 mojian 项目」错误路径、重复初始化拒绝，
//! 以及 run/decide 桩输出。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 建一个唯一临时目录（不复用系统临时根，避免测试间串扰）。
fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mojian-cli-test-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// 以隔离的 `MOJIAN_HOME` 运行 `mojian <args...>`。
fn run_mojian(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mojian"))
        .env("MOJIAN_HOME", home)
        .args(args)
        .output()
        .expect("run mojian binary")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn combined(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn new_then_status_happy_path() {
    let home = unique_dir("home-happy");
    let proj = unique_dir("proj-happy").join("mybook");

    let out = run_mojian(&home, &["new", proj.to_str().unwrap()]);
    assert!(out.status.success(), "new 应成功：{}", combined(&out));

    let so = stdout(&out);
    assert!(so.contains("project_id:"), "stdout 应含 project_id：{so}");
    assert!(so.contains("style_sampling"), "stdout 应含初始 phase：{so}");
    assert!(
        so.contains(proj.to_str().unwrap()),
        "stdout 应含项目绝对路径：{so}"
    );

    // 部署与登记落地。
    assert!(home.join("central.db").is_file(), "central.db 应存在");
    assert!(proj.join("mojian.toml").is_file(), "mojian.toml 应存在");
    assert!(proj.join("CLAUDE.md").is_file(), "CLAUDE.md 应部署");
    assert!(
        proj.join("prompts/sop-1-style").is_dir(),
        "prompts/sop-1-style 应部署"
    );
    assert!(
        proj.join(".claude/agents").is_dir(),
        ".claude/agents 应部署"
    );
    assert!(!proj.join("spec.toml").exists(), "spec.toml 不属部署载荷");

    // status 正常路径。
    let out = run_mojian(&home, &["status", "--path", proj.to_str().unwrap()]);
    assert!(out.status.success(), "status 应成功：{}", combined(&out));
    assert!(
        stdout(&out).contains("style_sampling"),
        "status stdout 应含 phase：{}",
        stdout(&out)
    );

    cleanup(&home);
    cleanup(&proj);
}

#[test]
fn status_on_non_mojian_dir_errors() {
    let home = unique_dir("home-nonproj");
    let empty = unique_dir("empty-dir");

    let out = run_mojian(&home, &["status", "--path", empty.to_str().unwrap()]);
    assert!(!out.status.success(), "非 mojian 项目应非 0 退出");
    assert!(
        combined(&out).contains("非 mojian 项目"),
        "应含错误信息：{}",
        combined(&out)
    );

    cleanup(&home);
    cleanup(&empty);
}

#[test]
fn new_rejects_reinitialization() {
    let home = unique_dir("home-reinit");
    let proj = unique_dir("proj-reinit").join("mybook");

    let first = run_mojian(&home, &["new", proj.to_str().unwrap()]);
    assert!(first.status.success(), "首次 new 应成功：{}", combined(&first));

    let second = run_mojian(&home, &["new", proj.to_str().unwrap()]);
    assert!(!second.status.success(), "重复 new 应非 0 退出");

    cleanup(&home);
    cleanup(&proj);
}

#[test]
fn status_restores_tampered_spec() {
    let home = unique_dir("home-tamper");
    let proj = unique_dir("proj-tamper").join("mybook");

    let out = run_mojian(&home, &["new", proj.to_str().unwrap()]);
    assert!(out.status.success(), "new 应成功：{}", combined(&out));

    let claude = proj.join("CLAUDE.md");
    let mut content = std::fs::read_to_string(&claude).unwrap();
    content.push_str("\ntampered\n");
    std::fs::write(&claude, content).unwrap();

    let out = run_mojian(&home, &["status", "--path", proj.to_str().unwrap()]);
    assert!(out.status.success(), "篡改后 status 应成功：{}", combined(&out));

    let restored = std::fs::read_to_string(&claude).unwrap();
    assert!(
        !restored.contains("tampered"),
        "被篡改内容应被重部署覆盖还原：{restored}"
    );

    cleanup(&home);
    cleanup(&proj);
}

#[test]
fn run_and_decide_are_stubs() {
    let home = unique_dir("home-stub");

    let out = run_mojian(&home, &["run"]);
    assert!(out.status.success(), "run 桩应 exit 0");
    assert!(
        stdout(&out).contains("stub，将在 ITER-002 实现"),
        "run 应打印桩提示：{}",
        stdout(&out)
    );

    let out = run_mojian(&home, &["decide", "CH-001", "CONFIRMED"]);
    assert!(out.status.success(), "decide 桩应 exit 0（忽略尾随参数）");
    assert!(
        stdout(&out).contains("stub，将在 ITER-002 实现"),
        "decide 应打印桩提示：{}",
        stdout(&out)
    );

    cleanup(&home);
}

fn cleanup(dir: &Path) {
    std::fs::remove_dir_all(dir).ok();
}
