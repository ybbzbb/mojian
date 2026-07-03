# mojian（墨简）

**AI 辅助网文创作系统：三个 SOP + 一个执行器。**

宋代说书人靠一册话本在茶馆演出万言——mojian 让模型做同样的事：骨架先行，人审品味，程序管状态。写一本网文小说，被完整定义为依次执行三个标准作业程序（SOP）：

| | SOP | 一句话 |
|---|------|--------|
| ① | [风格抽取](https://github.com/ybbzbb/mojian/issues/5) | 从参考小说客观提炼可借鉴的结构，与作者讨论出创意愿景 |
| ② | [圣经·大纲构建](https://github.com/ybbzbb/mojian/issues/6) | 定义小说的基因——圣经一旦确认，整部小说就已存在 |
| ③ | [写作循环](https://github.com/ybbzbb/mojian/issues/7) | 批次流水线：骨架 → 人审 → 正文 → 人审，一致性交给代码，品味交给人 |

SOP 与提示词是本项目的核心资产，**全部开源**。程序（Rust 核心 + CLI + 桌面客户端）只是 SOP 的执行器：管状态、管调度、管统计，让创作可复现、可中断恢复、可被社区改进。

方法论提炼自一部 94 章 / 32 万字小说的真实创作实践，实测数据与完整设计见 **[设计总纲 RFC](https://github.com/ybbzbb/mojian/issues/1)**。

## 项目状态

**理念定型阶段。** 三个 SOP 的设计正在 issue 中讨论定稿，欢迎参与：

- 设计总纲：[#1](https://github.com/ybbzbb/mojian/issues/1)
- SOP 设计：[#5](https://github.com/ybbzbb/mojian/issues/5) · [#6](https://github.com/ybbzbb/mojian/issues/6) · [#7](https://github.com/ybbzbb/mojian/issues/7)

设计定稿后进入实现：`mojian-core`（状态机 + SQLite + 统计）→ `mojian` CLI → Tauri 桌面客户端。

## 参与

最欢迎的两类贡献：

1. **风格包**——不同题材（都市/玄幻/历史…）的提示词与写作指南
2. **失败模式**——真实创作中踩过的坑，沉淀为全链路的避坑清单

## License

[Apache-2.0](LICENSE)
