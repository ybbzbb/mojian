# Tech Design · Domain Model

> 领域实体与两级状态机（设计草案，planned）。命名对齐 `docs/naming.md` 已固化的枚举变体。

## 两级状态模型

把 ink_node 那条从 `needs_init` 一路拉到章节 status 的 20+ 节点单链**拆成两级**：粗粒度是 SOP 阶段，细粒度是每个 SOP 内部的状态机（形态各 SOP 不同）。

### Level 1 — SOP 阶段（粗）

三个 SOP 各自一段，不首尾相连成一条长链：

```
SOP①  style_sampling → style_extracting(块游标) → brief_drafting →[关卡:brief]
                                                 → vision_drafting →[关卡:vision]
SOP②  bible_building → bible_check(程序) → bible_verify(LLM) →[关卡:圣经]
                     → outline_expand → outline_verify →[关卡:大纲]
SOP③  按卷循环（内部见 Level 2）
```

### Level 2 — SOP 内部状态机（细）

- **SOP① 抽取游标**（每参考书、每 ~5 万字块）：`pending → extracting(块) → extracted`，断点续抽。
- **SOP③ 章节状态机**（最细一层，以「批次」为调度单位）：

```
planned → skeleton_drafting → skeleton_review → prose_drafting → prose_review → approved
                                   │                                 │
                                   └──(REVISE 打回)                  └── void → planned
```

对齐 `naming.md`：`Planned / SkeletonDrafting / SkeletonReview / ProseDrafting / ProseReview / Approved / Void`。

### 与 ink_node 的关键简化

ink_node 每章有 `skeleton_verifying`（LLM 逐章验证骨架）。mojian 按 #7「LLM 审查只留卷边界」**砍掉逐章 LLM 验证器**——骨架的客观项由 Rust 检查器把关，`skeleton_review` 直接是**批量人工关卡**。少一个状态、少一次冷启动、少一批 token（AP-003 落地）。

## 领域实体（概念层，字段详见 `storage.md`）

> 下列实体的机器状态**全部存在客户端中央 DB**（按 `project_id` 分区），不在项目目录里。项目目录只是运行环境（SSOT + SPEC 缓存）。作用域见 `overview.md`。

| 实体 | 说明 | 关键状态/属性 |
|---|---|---|
| Project | 一个小说项目（= 一个运行环境目录） | 当前 SOP phase、当前卷、游标、部署 SPEC 版本 |
| ReferenceBook | 参考小说 | 抽取游标（块级） |
| Volume（卷/Arc） | SOP③ 的循环单位 | arc phase |
| Batch（批次） | SOP③ 的调度单位（每批 3-5 章） | 批状态 |
| Chapter（章节） | 最细的状态机载体 | status、verify_flag、deviation |
| BibleVersion | 圣经的版本化记录 | 版本、原因、触发源 |
| VoidRecord | 作废记录 | 章节、原因、影响范围 |
| Decision | 人在关卡的决定 | 判定、评论/补充信息 |

## VOID 机制（SPEC 定义，程序只记录）

**语义在 SPEC**：什么时候 void、怎么修圣经、级联到哪，写在 SOP② 的 SPEC 里，由人 + 提示词判断。执行器**不设计**这套逻辑。

**程序只做没脑子的部分**：

- 圣经改动时写 `bible_version`（版本、原因、触发源=void/人工）。
- 记 `void_record`（章节、原因、影响范围）。
- **过期检测**：`generation_log` 记录了每章上次生成读的输入切片及其 hash。圣经改动后新旧 hash 不同，「哪些章节的输入过期了」就是一句免费的 DB 查询 → 程序**标记候选受影响章节**，但**是否 void 由人 + SPEC 定**。
- 按 SPEC 指令把受影响章节 `status: void → planned`。

## 开放问题

1. 批次大小：固定 3-5 章，还是按人工审阅吞吐自适应？（#7 开放问题①）
2. 批内并行、批间串行的取舍：正文是否严格串行（前章 approved 才写下一章）？（#7 开放问题③）
