---
name: esr-naming
description: >
  Use when naming classes, interface parameters, business concepts, or types
  in an ESR platform project — provides a business-concept naming dictionary
  (Chinese term -> English name) plus the *Param / *DTO / *VO layering and
  platform-level code naming conventions. Complements project-level
  docs/naming.md (which owns project-specific naming).
---

# ESR 平台命名规范

本 skill 覆盖两块内容（并存，互不替代）：

1. **业务概念命名字典**：中文业务名 → 英文命名的结构化 mapping 表，保证同一业务概念全平台只有一个英文命名。
2. **代码结构命名约定**：类后缀、参数分层、注释风格、分页惯例等平台级约定。

与 `docs/naming.md` 的边界见文末「边界声明」。

---

## 一、业务概念命名字典

> 表为**可持续增补**的命名字典。新增业务概念时只追加行，不改既有行（除非人工裁决勘误）。一个中文业务概念在全平台**只有一个**英文命名，跨字段/类/接口统一使用。与 mapping 冲突的历史命名视为反例，新代码不得沿用。

| 中文业务名 | 英文命名 | 说明 |
|-----------|---------|------|
| 国家 | `country` | **禁止**用 `site` / `source` 等替代 |
| 国家编码 | `countryCode` | |
| 品类 | `category` | 注：历史代码中出现 `catery` 系笔误，统一修正 |
| 产品一级分类 | `categoryLevel1` | 采用 `categoryLevelN` 形态；禁用 `catery1` / `catery2` 写法 |
| 产品二级分类 | `categoryLevel2` | |
| 产品三级分类 | `categoryLevel3` | |
| 属性 | `attribute` | |
| 机型 | `model` | |
| 赛道 | `track` | |
| SellerSKU / msku | `sellerSku` | |

---

## 二、代码结构命名约定

### 接口参数三层分层

| 后缀 | 含义 | 用法 |
|------|------|------|
| `*Param` | 接口**入参**对象 | 查询/分页入参；分页入参 `extends Query`（如 `CaseOrderRefundPageParam extends Query`）；字段加 `@Schema(description=...)` |
| `*DTO` | **命令/传输**对象 | 写操作入参，承载校验逻辑；支持 `@Validated` 分组（如 `@Validated(ModifyTagDTO.Modify.class)`）；用于 add / update / batch 等操作 |
| `*VO` | 接口**出参**对象 | 响应返回；列表分页统一 `PageInfo<XxxVO>`（如 `CaseOrderRefundPageVO`、`CodeVO`） |

### 其他平台级代码命名

| 形态 | 约定 |
|------|------|
| `*Controller` | REST 控制器，`extends BaseController`，标注 `@RestController` |
| `*Service` / `*ServiceImpl` | 业务服务接口 / 实现类 |
| `*Mapper` / `*Dao` | 持久层接口 |
| `*Entity` / `*DO` | 数据库实体对象 |
| `*Enum` / `*CodeEnum` | 枚举 / 错误码枚举（如 `BaseCodeEnum`） |

### 字段文档约定

- 使用 `@Schema(description=...)` 标注字段含义，不写冗余 Javadoc 注释。
- 示例：`@Schema(description = "国家编码") private String countryCode;`

### 分页约定

- 入参：`extends Query`（继承平台基础查询类）
- 出参：`PageInfo<XxxVO>`
- 端点：`POST /page`（分页查询统一使用 POST）

---

## 边界声明（REQ-021 / CON-006）

| 规范文件 | 管辖范围 |
|---------|---------|
| **本 skill（esr-naming，平台级）** | ESR 平台业务概念命名字典（中英 mapping）+ 跨 ESR 项目通用的代码命名后缀分层、`@Schema`、分页结构等惯例 |
| **`docs/naming.md`（项目级）** | 具体业务项目的文件名风格、commit message 格式、本项目特有字段命名等 |

衔接方式：`docs/naming.md` 通过一句引用指向本 skill（"业务概念命名与接口参数分层见 esr-naming skill"），两者**内容不重复**。

扩展方向：业务概念命名字典随业务推进持续加行；代码结构命名约定可增 ESR 特有的事件/任务/缓存键命名等。
