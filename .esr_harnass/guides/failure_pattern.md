# Guide: docs/failure_pattern.md

本文件说明如何生成业务项目的 `docs/failure_pattern.md`。

## 目标

沉淀一份让 sub-agent 能直接理解和规避的失败模式知识库。

## 推荐结构

- `Anti-Patterns`
- `Recurring Bugs`

每条都应尽量包含：

- 现象
- 根因
- 正确做法或预防措施
- 可选：首次发现时间、相关提交、复发条件

## 写法要求

- 必须可操作，不能写“注意一点”这种空话
- 只记录已经发生过、或已经被团队确认值得防范的问题
- 关注 agent 真会踩到的坑：依赖、环境、分层、命名、测试假象、数据一致性
- 如果当前项目还没有沉淀内容，可以先保留章节并注明待补充

## 对 agent 的价值

- builder-agent 在实现前据此避坑
- archivist-agent 可在复盘后建议人工补充

## 示例骨架

以下展示 `docs/failure_pattern.md` 的典型结构：

```markdown
# Failure Patterns

## Anti-Patterns

### AP-001 跨层直接调用

- 现象：Service 层直接调用另一模块的 Mapper，绕过对方 Service。
- 根因：赶进度时图方便，忽略分层边界。
- 正确做法：通过被调模块的 Service 接口访问，不直接跨层。

### AP-002 在 Controller 写业务逻辑

- 现象：权限校验、数据聚合逻辑写在 Controller 方法体内。
- 根因：认为 Controller 是"入口"，就地处理更快。
- 正确做法：Controller 只做参数接收和响应组装，业务逻辑移至 Service。

## Recurring Bugs

### BUG-001 分页参数未传导致全量查询

- 现象：列表接口偶发超时，日志显示 SQL 无 LIMIT。
- 根因：调用方未传 pageNum/pageSize，Service 未做默认值兜底。
- 预防：Service 入参校验 pageSize 非空且 ≤ 100；Mapper 统一走 `PageHelper.startPage`。
- 首次发现：ITER-003（issue#7）
```
