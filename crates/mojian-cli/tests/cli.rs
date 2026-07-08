//! `mojian` 端到端集成测试：以 `MOJIAN_HOME` 指向隔离临时目录，驱动真实二进制。
//!
//! 覆盖 new→status 正常路径、「非 mojian 项目」错误路径、重复初始化拒绝，
//! 以及 run→decide→run 端到端（mock SDK 经 MOJIAN_CLAUDE_CMD 注入，不触达真实 claude）。

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

/// 以隔离 `MOJIAN_HOME` + 注入的假生成命令运行 `mojian <args...>`（不触达真实 claude）。
fn run_mojian_sdk(home: &Path, fake_cmd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mojian"))
        .env("MOJIAN_HOME", home)
        .env("MOJIAN_CLAUDE_CMD", fake_cmd)
        .args(args)
        .output()
        .expect("run mojian binary")
}

/// 写一个打印固定 `claude --output-format json` 形状 JSON 的假命令脚本（可执行）。
fn write_fake_sdk(dir: &Path) -> PathBuf {
    let path = dir.join("fake-claude.sh");
    std::fs::write(
        &path,
        "#!/bin/sh\ncat <<'JSON'\n{\"result\":\"占位 brief 产出\",\"total_cost_usd\":0.0123,\"usage\":{\"input_tokens\":1200,\"output_tokens\":800}}\nJSON\n",
    )
    .expect("write fake sdk script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).unwrap();
    }
    path
}

/// 从 `mojian new` 的 stdout 提取 `project_id`。
fn project_id_from(new_stdout: &str) -> String {
    new_stdout
        .lines()
        .find_map(|line| line.strip_prefix("project_id:"))
        .map(|rest| rest.trim().to_string())
        .expect("new stdout 应含 project_id")
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
fn run_decide_run_end_to_end() {
    let home = unique_dir("home-e2e");
    let proj = unique_dir("proj-e2e").join("mybook");
    let fake_dir = unique_dir("fake-sdk");
    let fake = write_fake_sdk(&fake_dir);
    let proj_arg = proj.to_str().unwrap();

    // new：建项目。
    let out = run_mojian(&home, &["new", proj_arg]);
    assert!(out.status.success(), "new 应成功：{}", combined(&out));
    let project_id = project_id_from(&stdout(&out));
    let gen_log = home.join("logs").join(&project_id).join("generation.jsonl");
    let dec_log = home.join("logs").join(&project_id).join("decision.jsonl");

    // 首次 run：装配 → 调假 SDK → 写 generation.jsonl → 停在 brief 关卡。
    let out = run_mojian_sdk(&home, &fake, &["run", "--path", proj_arg]);
    assert!(out.status.success(), "首次 run 应成功：{}", combined(&out));
    assert!(
        stdout(&out).contains("brief"),
        "run 应停在 brief 关卡：{}",
        stdout(&out)
    );
    assert!(gen_log.is_file(), "generation.jsonl 应生成");
    let gen1 = std::fs::read_to_string(&gen_log).unwrap();
    let line1 = gen1.lines().last().expect("generation.jsonl 至少一行");
    for field in ["step", "agent", "token_in", "token_out", "cost"] {
        assert!(line1.contains(field), "首次 generation 行应含 {field} 字段：{line1}");
    }

    // status：显示卡在 brief 关卡（REQ-008）。
    let out = run_mojian(&home, &["status", "--path", proj_arg]);
    assert!(out.status.success(), "status 应成功：{}", combined(&out));
    let so = stdout(&out);
    assert!(so.contains("project"), "status 应含 project：{so}");
    assert!(so.contains("phase"), "status 应含 phase：{so}");
    assert!(so.contains("卡在 brief 关卡"), "status 应显示卡点：{so}");

    // decide brief REVISE --comment：写 decision.jsonl + 回退。
    let out = run_mojian(
        &home,
        &["decide", "brief", "REVISE", "--comment", "钩子太弱", "--path", proj_arg],
    );
    assert!(out.status.success(), "decide REVISE 应成功：{}", combined(&out));
    assert!(dec_log.is_file(), "decision.jsonl 应生成");
    let dec = std::fs::read_to_string(&dec_log).unwrap();
    assert!(
        dec.contains("brief") && dec.contains("REVISE") && dec.contains("钩子太弱"),
        "decision.jsonl 应含 gate/verdict/comment：{dec}"
    );

    // REQ-011 回喂：再次 run，新 generation 行 inputs 含上一轮评论。
    let out = run_mojian_sdk(&home, &fake, &["run", "--path", proj_arg]);
    assert!(out.status.success(), "回喂后 run 应成功：{}", combined(&out));
    let gen2 = std::fs::read_to_string(&gen_log).unwrap();
    assert!(gen2.lines().count() >= 2, "generation.jsonl 应新增一行：{gen2}");
    let line2 = gen2.lines().last().expect("应有新行");
    assert!(
        line2.contains("钩子太弱"),
        "新 generation 行 inputs 应含回喂评论：{line2}"
    );

    // decide brief CONFIRMED：推进（brief → vision_drafting）。
    let out = run_mojian(&home, &["decide", "brief", "CONFIRMED", "--path", proj_arg]);
    assert!(out.status.success(), "decide CONFIRMED 应成功：{}", combined(&out));

    // 再次 run：不再卡在 brief，通路成立（REQ-012）。
    let out = run_mojian_sdk(&home, &fake, &["run", "--path", proj_arg]);
    assert!(out.status.success(), "推进后 run 应成功：{}", combined(&out));
    assert!(
        !stdout(&out).contains("卡在 brief"),
        "推进后不应再卡在 brief：{}",
        stdout(&out)
    );

    // 错误路径：当前无关卡时 decide brief 关卡不匹配，非 0 退出、不 panic。
    let out = run_mojian(&home, &["decide", "brief", "CONFIRMED", "--path", proj_arg]);
    assert!(!out.status.success(), "关卡不匹配应非 0 退出");
    assert!(
        combined(&out).contains("关卡状态不匹配"),
        "应含关卡不匹配错误：{}",
        combined(&out)
    );

    cleanup(&home);
    cleanup(&proj);
    cleanup(&fake_dir);
}

fn cleanup(dir: &Path) {
    std::fs::remove_dir_all(dir).ok();
}
