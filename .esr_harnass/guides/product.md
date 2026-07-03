# Guide: docs/product.md

本文件说明如何生成业务项目的 `docs/product.md`。

## 目标

输出一份给人和 agent 都能读懂的当前产品快照，描述产品边界、用户、已有能力与非目标。

## 必须基于

- `README.md`
- 当前代码结构与路由/API
- 已存在的业务文档

## 必备章节

- `Product Overview`
- `Target Users`
- `Current Features`
- `Non-Goals`
- `Known Gaps / Open Questions`

## 写法要求

- 写当前事实，不写希望或口号
- `Current Features` 只写仓库里已经能证明存在的能力
- `Non-Goals` 要能收边界，避免 requirements-agent 无限制扩需求
- 不清楚的内容写 `TODO`

## 对 agent 的价值

- requirements-agent 用它判断需求是否超出产品边界
- archivist-agent 用它在迭代关闭时归档真实变化

## 示例骨架

以下展示 `docs/product.md` 的典型结构：

```markdown
# Product

## Product Overview

一句话描述产品是什么：面向 XX 业务团队的 XX 管理平台，支持 XX 和 XX 核心能力。

## Target Users

- 主要用户：XX 运营团队（日常管理操作）
- 次要用户：XX 数据分析师（报表查询）

## Current Features

- 订单管理：列表查询、详情查看、状态流转（待确认→已确认→已完成）
- 退款管理：退款申请、审批、金额核算
- 报表导出：按日期范围导出订单明细 CSV

## Non-Goals

- 不做 C 端用户界面（纯 B 端后台）
- 不做支付能力（支付对接由独立支付服务负责）
- 不做实时推送通知（暂无 WebSocket 计划）

## Known Gaps / Open Questions

- 权限体系当前为全员统一权限，按角色细分权限为 TODO
- 报表维度单一，多维度交叉分析为 TODO
```
