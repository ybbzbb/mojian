type: guide
doc_version: v1
topic: prd-checklist

## 写作指引

本文件指导业务项目 AI 按此 guide 生成 `docs/prd/checklist.md`。

`docs/prd/checklist.md` 是 prd-gatekeeper-agent 用于判断 PRD 结构完整性的门禁规则文件。

**核心定位：只判 PRD 结构是否完整，不评价内容质量。**

- 通过条件只看"是否声明了某项内容"，不看声明的内容是否正确或合理
- 实际项目使用时，由人编辑替换最小维度，按项目需要增减检查项
- 本 guide 提供的模板仅覆盖三个最小维度作为骨架示例

生成 `docs/prd/checklist.md` 时，直接复用下方模板正文，删除本写作指引节。

---

## 模板正文

# PRD Checklist

version: v1
updated: YYYY-MM-DD

| 维度 | 检查项 | 通过条件 |
|------|--------|----------|
| 验收标准 | PRD 中是否包含可验证的验收标准 | PRD 中存在显式标注的验收章节或验收条目（如"验收标准"、"AC"、"Acceptance Criteria"等标题或列表） |
| 非目标 | PRD 中是否声明了本次不做的事项 | PRD 中存在显式的非目标声明（如"Non-Goals"、"不包含"、"超出范围"等标题或列表） |
| 用户场景 | PRD 中是否描述了目标用户与使用场景 | PRD 中存在对目标用户群体及其核心使用场景的描述（如"用户故事"、"使用场景"、"Target Users"等） |
