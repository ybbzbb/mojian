# Guide: docs/naming.md

本文件说明如何生成业务项目的 `docs/naming.md`。

## 目标

把项目当前实际采用的命名与编码规则写清楚，让 builder-agent 在实现时有统一约束。

## 必备章节

- `File / Module Naming`
- `Function / Variable Naming`
- `API Naming`
- `Commit Message`
- `注释规则`

## 写法要求

- 优先总结仓库现状，其次才是补充规范
- 规则必须能落到文件名、字段名、函数名等具体对象上
- 禁止只有抽象口号，没有例子

## 对 agent 的价值

- builder-agent 依据它保持实现风格一致
- planning-agent 可据此生成更清晰的任务命名

## 示例骨架

以下展示 `docs/naming.md` 的典型结构：

```markdown
# Naming

## File / Module Naming

- Controller：`XxxController.java`（PascalCase）
- Service 接口：`XxxService.java`；实现：`XxxServiceImpl.java`
- Mapper：`XxxMapper.java`（对应 `XxxMapper.xml`）
- 前端页面：`kebab-case.vue`（如 `order-list.vue`）

## Function / Variable Naming

- 方法名：`camelCase`，动词开头（如 `getOrderById`、`createRefund`）
- 常量：`UPPER_SNAKE_CASE`（如 `MAX_RETRY_COUNT`）
- 局部变量：`camelCase`，语义明确（避免 `tmp`、`data`）

## API Naming

- REST 端点：`/资源复数/{id}/子资源`（如 `/orders/{orderId}/refunds`）
- 分页端点统一：`POST /xxx/page`
- 入参后缀：`*Param`（查询）、`*DTO`（命令）
- 出参后缀：`*VO`；分页出参：`PageInfo<XxxVO>`

## Commit Message

格式：`{type}: {description}`（英文，动词原形开头）

- `feat`: 新功能
- `fix`: bug 修复
- `refactor`: 重构（无功能变更）
- `docs`: 文档变更

## 注释规则

- 不写解释"做了什么"的注释；只在逻辑非常规时写注释说明"为什么"。
- 接口字段用 `@Schema(description=...)` 代替 Javadoc 注释。
```
