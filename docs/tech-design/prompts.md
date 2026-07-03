# Tech Design · Prompts (SPEC)

> SPEC 布局、加载、覆盖链、输入契约 manifest（设计草案，planned）。SPEC 定义见 `overview.md`。

## 布局

```
prompts/
├── sop-1-style/       # 风格抽取的各步骤 SPEC + 写作指南
├── sop-2-bible/       # 圣经·大纲构建的各步骤 SPEC
└── sop-3-writing/     # 写作循环的各步骤 SPEC（卷计划/品味校准/骨架/正文/卷末收口）
```

一个 SOP 步骤对应一份 SPEC 文件。SPEC 是**运行时资产**：明文、运行时加载、项目级可覆盖；不得把提示词硬编码进 Rust 源码。

## 覆盖链

加载某步骤 SPEC 时，按优先级解析：

1. 项目本地 `prompts/`（作者覆盖）
2. 框架自带默认

项目级覆盖让不同题材（都市/玄幻/历史…）能替换风格包与写作指南，而不改执行器。这是本项目最欢迎的两类贡献之一（另一类是失败模式清单）。

## 输入契约 manifest（切片的来源）

每个步骤在 SPEC 里声明它要读的最小切片，用**符号引用**而非死路径。示例（骨架步骤）：

```yaml
inputs:
  - bible.rules                 # 整文件
  - bible.style#skeleton        # style.md 里名为 skeleton 的那一段（段级切片）
  - bible.protagonist
  - outline.arc:{arc_id}
  - plan.chapters:{batch}       # 本批章节计划
  - prev_skeleton:{ch-1}
  - failure_pattern.active
  - foreshadowing.due:{arc_id,batch}
write:
  - chapter.skeleton:{batch}
```

- 装配器（见 `engine.md`）把 `{arc_id}` `{batch}` 等符号按当前 DB 状态解析成具体文件路径 + 段落抽取 + 内容 hash。
- `write:` 声明喂给 `bundle.write_scope`（沙箱写白名单）。
- `#anchor` 段级切片依赖 SSOT 文件的稳定小标题锚点（见 `storage.md` 开放问题②）。

## step → SPEC 映射

状态机的每个「非人工」动作对应唯一一份 SPEC + 一份输入契约。映射表由执行器持有（planned；随状态机定稿）。VOID 的语义也写在对应 SOP 的 SPEC 里（程序只记录，见 `domain-model.md`）。

## 与 ink_node 的对照

ink_node 的 16 个 agent 合同（`agents/*.md`）+ guides 就是前身的 SPEC。mojian 按真实价值归并为三个 SOP 的提示词包，并把编排/切片/状态从 SPEC（LLM 职责）下沉到 Rust（程序职责）——SPEC 只负责「生成/分析」，不负责流程控制。

## 开放问题

1. 输入契约 manifest 放 SPEC frontmatter 还是 sidecar 文件？（影响 SPEC 可读性 vs 解析简洁性）
2. 符号引用的语法（`#anchor` / `:{param}`）需与 SSOT 锚点约定一起定稿。
