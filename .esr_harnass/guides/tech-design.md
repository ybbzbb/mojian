# Guide: docs/tech-design.md

本文件说明如何生成业务项目的 `docs/tech-design.md`。

## 目标

输出项目级技术设计基线，供 design-agent、planning-agent、builder-agent 和 archivist-agent 共同参考。

## 必须基于

- 仓库目录结构
- 已存在的服务边界、框架、依赖、部署方式
- 已知的架构约束

## 必备章节

- `System Architecture`
- `Module Responsibilities`
- `Tech Stack`
- `Architectural Constraints`
- `Adding New Dependencies`

## 写法要求

- 只写项目级稳定约束，不写单次迭代方案
- `Architectural Constraints` 必须可执行，不能写空话
- 技术栈版本与边界要尽量具体
- 若仓库无法证明某项结论，写 `TODO`

## 对 agent 的价值

- design-agent 用它避免提出越界方案
- builder-agent 用它判断是否允许新增依赖或跨层实现
- archivist-agent 用它归档迭代造成的项目级架构变化

## 示例骨架

以下展示 `docs/tech-design.md` 的典型结构：

```markdown
# Tech Design

## System Architecture

单体 Spring Boot 应用，部署在 2 台服务器（见 infra.md）；MySQL 8.0 作为主数据库，Redis 6 作缓存。无微服务拆分计划。

## Module Responsibilities

| 模块 | 职责 |
|------|------|
| `order` | 订单生命周期管理，状态机流转 |
| `refund` | 退款申请与审批流程 |
| `report` | 报表查询与导出（只读，不写主库） |
| `common` | 工具类、异常、统一响应结构 |

## Tech Stack

| 层 | 技术 | 版本 |
|----|------|------|
| 框架 | Spring Boot | 2.7.x |
| ORM | MyBatis-Plus | 3.5.x |
| 数据库 | MySQL | 8.0 |
| 缓存 | Redis | 6.x |
| 构建 | Maven | 3.8.x |
| JDK | OpenJDK | 11 |

## Architectural Constraints

- 禁止跨模块直接调用 Mapper；必须通过对方 Service 接口。
- Controller 不写业务逻辑；业务逻辑写在 Service。
- 新接口必须走 `BaseController` + `@EnableResponseWrapper` 统一响应包装。
- 禁止在 Service 层直接返回数据库 Entity；必须转换为 VO/DTO。

## Adding New Dependencies

新增 Maven 依赖必须经过人工确认，并在迭代 tech-design.md 中声明选型理由。禁止 builder-agent 自行引入 tech-design.md 未声明的依赖。
```
