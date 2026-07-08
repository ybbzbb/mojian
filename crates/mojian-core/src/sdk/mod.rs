use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use crate::error::CoreError;

mod claude_cli;

pub use claude_cli::ClaudeCliRunner;

/// 一次 SDK 调用喂进去的完整上下文（对齐 engine.md「bundle 五字段」+ REQ-003）。
/// 由 `context::assemble_bundle` 组装，交给 `GenerationRunner::run` 消费。
#[derive(Debug, Clone)]
pub struct Bundle {
    /// 部署 agent 的相对路径（如 `.claude/agents/brief-agent.md`）。
    pub agent: String,
    /// 本步 SPEC 切片（提示词 / 步骤说明）。
    pub spec_slice: String,
    /// 切片后的 SSOT 结构化参数，内联本步所需段落 / 条目，并含 decision.jsonl 回喂的人类评论。
    pub inputs: String,
    /// 写白名单：由 manifest `write` 推导，逐项作为 `--add-dir` 传入子进程。
    pub write_scope: Vec<PathBuf>,
    /// 期望产出与 done 信号形状（输出契约）。
    pub output_contract: String,
}

/// SDK 生成命令的抽象。默认实现 `ClaudeCliRunner` 走无头 `claude` 子进程；测试可注入
/// 不 spawn 进程的假实现，满足「外部命令必须可替换」硬约束。
pub trait GenerationRunner {
    fn run(&self, bundle: &Bundle) -> Result<SdkResponse, CoreError>;
}

/// 解析 `claude --output-format json` stdout 得到的结构化结果（REQ-005）。
/// `cost` 映射顶层 `total_cost_usd`；`usage_in` / `usage_out` 映射嵌套
/// `usage.input_tokens` / `usage.output_tokens`。字段缺失时以 `Option::None` 容错，不报错。
#[derive(Debug, Clone)]
pub struct SdkResponse {
    pub result: String,
    pub cost: Option<f64>,
    pub usage_in: Option<u64>,
    pub usage_out: Option<u64>,
}

impl<'de> Deserialize<'de> for SdkResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawUsage {
            #[serde(default)]
            input_tokens: Option<u64>,
            #[serde(default)]
            output_tokens: Option<u64>,
        }

        #[derive(Deserialize)]
        struct Raw {
            result: String,
            #[serde(rename = "total_cost_usd", default)]
            cost: Option<f64>,
            #[serde(default)]
            usage: Option<RawUsage>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let (usage_in, usage_out) = match raw.usage {
            Some(usage) => (usage.input_tokens, usage.output_tokens),
            None => (None, None),
        };

        Ok(SdkResponse {
            result: raw.result,
            cost: raw.cost,
            usage_in,
            usage_out,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 不 spawn 任何进程的假实现：回放注入的固定响应，供 core 单元测覆盖 trait 注入路径。
    struct FakeRunner {
        response: SdkResponse,
    }

    impl GenerationRunner for FakeRunner {
        fn run(&self, _bundle: &Bundle) -> Result<SdkResponse, CoreError> {
            Ok(self.response.clone())
        }
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
    fn sdk_response_parses_full_json() {
        let json = r#"{"result":"占位 brief","total_cost_usd":0.01,"usage":{"input_tokens":10,"output_tokens":20}}"#;
        let resp: SdkResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.result, "占位 brief");
        assert_eq!(resp.cost, Some(0.01));
        assert_eq!(resp.usage_in, Some(10));
        assert_eq!(resp.usage_out, Some(20));
    }

    #[test]
    fn sdk_response_tolerates_missing_cost_and_usage() {
        let json = r#"{"result":"仅有结果"}"#;
        let resp: SdkResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.result, "仅有结果");
        assert!(resp.cost.is_none());
        assert!(resp.usage_in.is_none());
        assert!(resp.usage_out.is_none());
    }

    #[test]
    fn sdk_response_tolerates_partial_usage() {
        let json = r#"{"result":"半份用量","usage":{"input_tokens":42}}"#;
        let resp: SdkResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.usage_in, Some(42));
        assert!(resp.usage_out.is_none());
        assert!(resp.cost.is_none());
    }

    #[test]
    fn fake_runner_returns_injected_response_without_spawn() {
        let runner = FakeRunner {
            response: SdkResponse {
                result: "假实现产出".to_string(),
                cost: Some(0.05),
                usage_in: Some(7),
                usage_out: Some(9),
            },
        };
        let resp = runner.run(&sample_bundle()).unwrap();
        assert_eq!(resp.result, "假实现产出");
        assert_eq!(resp.cost, Some(0.05));
        assert_eq!(resp.usage_in, Some(7));
        assert_eq!(resp.usage_out, Some(9));
    }
}
