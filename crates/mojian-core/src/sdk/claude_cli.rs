use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::error::CoreError;
use crate::sdk::{Bundle, GenerationRunner, SdkResponse};

const CLAUDE_CMD_ENV: &str = "MOJIAN_CLAUDE_CMD";
const DEFAULT_CLAUDE_CMD: &str = "claude";

/// 默认生成命令实现：以 `std::process::Command` 拼无头 `claude` 调用（REQ-004）。
/// 基础命令名读自 `MOJIAN_CLAUDE_CMD`（缺省 `claude`），据此可注入测试假命令而不触达真实
/// claude；子进程在项目目录内运行，抓 stdout JSON 解析为 `SdkResponse`。
pub struct ClaudeCliRunner {
    project_dir: PathBuf,
}

impl ClaudeCliRunner {
    pub fn new(project_dir: impl Into<PathBuf>) -> Self {
        Self {
            project_dir: project_dir.into(),
        }
    }

    fn base_command() -> String {
        env::var(CLAUDE_CMD_ENV).unwrap_or_else(|_| DEFAULT_CLAUDE_CMD.to_string())
    }
}

/// 把 bundle 五字段拼成喂给 `claude -p` 的提示词。确定性文本拼装，无随机 / 无环境依赖。
fn compose_prompt(bundle: &Bundle) -> String {
    format!(
        "# Agent\n{}\n\n# Spec\n{}\n\n# Inputs\n{}\n\n# Output Contract\n{}",
        bundle.agent, bundle.spec_slice, bundle.inputs, bundle.output_contract
    )
}

impl GenerationRunner for ClaudeCliRunner {
    fn run(&self, bundle: &Bundle) -> Result<SdkResponse, CoreError> {
        let base_cmd = Self::base_command();
        let prompt = compose_prompt(bundle);

        let mut command = Command::new(&base_cmd);
        command
            .current_dir(&self.project_dir)
            .arg("-p")
            .arg(&prompt)
            .arg("--output-format")
            .arg("json")
            .arg("--allowedTools")
            .arg("Read,Write,Edit");
        for dir in &bundle.write_scope {
            command.arg("--add-dir").arg(dir);
        }

        let output = command.output().map_err(|source| CoreError::Io {
            path: PathBuf::from(&base_cmd),
            source,
        })?;

        if !output.status.success() {
            return Err(CoreError::SubprocessFailed {
                command: base_cmd,
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let response: SdkResponse = serde_json::from_str(stdout.trim())?;
        Ok(response)
    }
}
