use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use mojian_core::{Bundle, ClaudeCliRunner, CoreError, GenerationRunner};

/// `MOJIAN_CLAUDE_CMD` 是进程级全局状态；集成测试在同一二进制内并行执行，须串行化
/// 对该环境变量的读写，避免测试互相踩踏。
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// 在隔离临时目录内生成一个可执行假命令脚本，返回其绝对路径。脚本正文由调用方给定，
/// `ClaudeCliRunner` 会真实 spawn 它以验证「外部命令可替换」硬约束，不触达真实 claude。
fn write_fake_command(name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mojian-sdk-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, body).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }

    path
}

fn sample_bundle() -> Bundle {
    Bundle {
        agent: ".claude/agents/brief-agent.md".to_string(),
        spec_slice: "撰写创作意图 brief".to_string(),
        inputs: "题材：都市悬疑".to_string(),
        write_scope: vec![PathBuf::from("creative")],
        output_contract: "creative/creative-brief.md".to_string(),
    }
}

#[test]
fn spawns_injected_command_and_parses_fixed_json() {
    let _guard = ENV_LOCK.lock().unwrap();

    let script = write_fake_command(
        "fake-claude-ok.sh",
        "#!/bin/sh\ncat <<'JSON'\n{\"result\":\"占位创作物 brief\",\"total_cost_usd\":0.0123,\"usage\":{\"input_tokens\":128,\"output_tokens\":256}}\nJSON\n",
    );
    std::env::set_var("MOJIAN_CLAUDE_CMD", &script);

    let runner = ClaudeCliRunner::new(std::env::temp_dir());
    let resp = runner.run(&sample_bundle()).unwrap();

    assert_eq!(resp.result, "占位创作物 brief");
    assert_eq!(resp.cost, Some(0.0123));
    assert_eq!(resp.usage_in, Some(128));
    assert_eq!(resp.usage_out, Some(256));

    std::env::remove_var("MOJIAN_CLAUDE_CMD");
}

#[test]
fn non_zero_exit_returns_subprocess_failed_without_panic() {
    let _guard = ENV_LOCK.lock().unwrap();

    let script = write_fake_command(
        "fake-claude-fail.sh",
        "#!/bin/sh\necho 'boom: claude 失败' >&2\nexit 3\n",
    );
    std::env::set_var("MOJIAN_CLAUDE_CMD", &script);

    let runner = ClaudeCliRunner::new(std::env::temp_dir());
    let err = runner.run(&sample_bundle()).unwrap_err();

    match err {
        CoreError::SubprocessFailed { code, stderr, .. } => {
            assert_eq!(code, Some(3));
            assert!(stderr.contains("boom"), "stderr 摘要应含子进程错误输出");
        }
        other => panic!("期望 SubprocessFailed，实际为 {other:?}"),
    }

    std::env::remove_var("MOJIAN_CLAUDE_CMD");
}
