---
name: esr-api-controller
description: >
  Use when writing or reviewing a REST Controller interface in a Spring
  (Spring MVC / Spring Boot) project on the ESR platform. Covers REST verb
  conventions, module+page interface splitting, @Validated checks, and ESR
  BaseController idioms. Do NOT use for non-Spring contexts (Flink jobs,
  shell scripts, pure-doc iterations).
---

# ESR API Controller 规范（仅适用于 Spring 项目）

## 1. 适用范围

本 skill 仅适用于基于 **Spring MVC / Spring Boot** 的 REST Controller 层开发与审查。

**不适用场景（非 Spring 上下文不应触发本 skill）：**

- Flink Job 类（无 Spring MVC Dispatcher 上下文）
- Shell 脚本
- 纯文档类迭代（无 Java 代码产出）

---

## 2. REST 动词约定

| 场景 | HTTP 方法 | 端点示例 |
|------|-----------|---------|
| 分页列表查询 | `POST` | `POST /page` |
| 单条查询 / 明细 | `GET` | `GET /{code}`、`GET /code` |
| 新增 | `POST` | `POST /` |
| 编辑（含打标、批量操作） | `PUT` | `PUT /updatetag`、`PUT /batchrefund` |
| 删除 / 作废 | `DELETE` | `DELETE /{codes}` |

> 注意：CSCN 历史代码中存在动词漂移（如将编辑写成 `POST /update`），本规范以上表为准；历史漂移仅作背景说明，不作范例。

---

## 3. 接口拆分原则

- 按**业务模块 + 页面**划分 Controller，不按数据库表划分。
  - 正例：`CaseOrderRefundController`（"退款单"业务模块）、`CaseListController`（Case 列表页）
  - 反例：`OrderController`（直接对应 `order` 表的全部 CRUD 入口）
- 每个接口对应一个明确的**需求 CASE** 或**页面交互**，避免一个接口承担多个页面场景。
- 同一业务模块内，建议按**页面角色**（列表页 / 详情页 / 编辑入口）组织接口，而非单纯按 CRUD 动词。

---

## 4. 校验约定

- 写操作入参统一加 `@Validated @RequestBody`，缺一不可。
- 支持**分组校验**：在 DTO 内定义 `interface Modify {}` / `interface BatchModify {}` 等分组接口，Controller 方法用 `@Validated(XxxDTO.Modify.class)` 精确指定分组。
- 字段级校验注解（`@NotEmpty`、`@NotNull`、`@Length` 等）写在 DTO 字段上，Controller 方法体内不手动判断字段有效性。
- 分页查询入参（`*Param extends Query`）若无强制字段，可不加 `@Validated`；若有业务约束字段，也可按需添加。

---

## 5. 注释约定

- **不写方法 Javadoc**（即 `/** ... */` 块注释）。
- 接口描述统一用 `@Operation(summary = "...")` 表达，由 Swagger / OpenAPI 渲染文档。
- 字段说明统一用 `@Schema(description = "...")` 写在 DTO / VO 字段上。
- 从上下文注入的内部字段（如 `companyCode`、`modifyCode`）加 `@Schema(description = "...", hidden = true)`，避免暴露给前端。
- 类级描述用 `@Tag(name = "模块名称")`。

---

## 6. ESR 平台惯用法

```java
@Tag(name = "官网退款单-订单退款")
@RestController
@RequestMapping("/case/order/refund")
@EnableResponseWrapper           // 框架自动包装返回值，方法直接返回业务对象
public class XxxController extends BaseController {
    // BaseController 提供三个上下文方法：
    //   getCompanyCode()  — 当前租户/公司编码
    //   getUser()         — 当前用户完整对象（含 jobNumber、name 等）
    //   getUserCode()     — 当前用户编码
}
```

**要点：**

- 必须 `extends BaseController`，禁止直接注入 `HttpServletRequest` 或 `SecurityContext` 来获取用户信息。
- `@EnableResponseWrapper` 由框架统一包装响应结构，Controller 方法**直接返回**业务类型（`Boolean`、`PageInfo<XxxVO>`、`List<XxxVO>` 等），不手写 `Result<T>`。
- 分页接口：入参 `XxxPageParam extends Query`（分页字段由 `Query` 基类携带），出参 `PageInfo<XxxVO>`，端点固定为 `POST /page`。
- `companyCode` 在 Controller 方法体内通过 `getCompanyCode()` 注入到 Param / DTO 的 `hidden` 字段，**不从前端传入**。

---

## 7. 命名分层

`*Param`、`*DTO`、`*VO` 的含义与使用场景，以及 ESR 平台业务概念命名字典（中文业务名 → 英文命名），参见 **esr-naming** skill。本文件不重复展开命名清单。

---

## 8. 真实示例（取自 CSCN `CaseOrderRefundController`，蒸馏后规范形态）

> **蒸馏说明——与原始 `CaseOrderRefundController` 的差异：**
>
> 1. **去 `@Deprecated`**：原始类标注了 `@Deprecated`（历史遗留，功能已迁出新模块），规范示例不以废弃类为范例，已移除。
> 2. **补 `@Validated`**：原始 `page()` 方法的 `@RequestBody` 未加 `@Validated`，不符合"写操作入参统一加校验"约定，示例已补齐。
> 3. **private → public**：原始 `findByCaseCode()` 方法访问修饰符为 `private`，Spring MVC 不路由私有方法，示例已改为 `public`。

```java
@Tag(name = "官网退款单-订单退款")
@RestController
@RequestMapping("/case/order/refund")
@EnableResponseWrapper
// 蒸馏差异①：原始类加了 @Deprecated，已移除
public class CaseOrderRefundController extends BaseController {

    private final CaseOrderRefundService service;

    public CaseOrderRefundController(CaseOrderRefundService service) {
        this.service = service;
    }

    // ── 单条 / 明细查询（GET）────────────────────────────────────────

    @Operation(summary = "根据Case Code 查询退款单")
    @GetMapping("/case/{caseCode}")
    // 蒸馏差异③：原始方法为 private（Spring 不路由），已改为 public
    public List<CaseOrderRefundPageVO> findByCaseCode(
            @Parameter(description = "code编码（,）分隔") @PathVariable String caseCode) {
        return service.findByCaseCode(caseCode);
    }

    @Operation(summary = "退款详情by-code")
    @GetMapping("/code")
    public OrderRefundVO getOrderRefundByCode(String code) {
        // getCompanyCode() 从 BaseController 继承，由框架注入，前端不传
        return service.getOrderRefundByCode(code, getCompanyCode());
    }

    // ── 分页列表查询（POST /page）────────────────────────────────────

    @Operation(summary = "官网退款单列表查询")
    @PostMapping("/page")
    // 蒸馏差异②：原始方法缺少 @Validated，已补齐
    public PageInfo<CaseOrderRefundPageVO> page(
            @Validated @RequestBody CaseOrderRefundPageParam param) {
        // companyCode 由 Controller 注入，不来自前端
        param.setCompanyCode(getCompanyCode());
        return service.page(param);
    }

    // ── 编辑（PUT）+ 分组校验 ────────────────────────────────────────

    @Operation(summary = "修改标签")
    @PutMapping("/updatetag")
    public Boolean updateTag(
            // ModifyTagDTO.Modify 分组：仅 codes 字段必填，labelKeys 无需传
            @Validated(ModifyTagDTO.Modify.class) @RequestBody ModifyTagDTO param) {
        param.setCompanyCode(getCompanyCode());
        param.setModifyCode(getUserCode());
        param.setModifyName(getUser().getName());
        return service.handleOneTag(param);
    }

    // ── 删除（DELETE）───────────────────────────────────────────────

    @Operation(summary = "退款-删除")
    @DeleteMapping("/{codes}")
    public Boolean delete(
            @Parameter(description = "code编码（,）分隔") @PathVariable String codes) {
        return service.delete(Arrays.asList(codes.replace("，", ",").split(",")));
    }
}
```

**关键点说明：**

- `CaseOrderRefundPageParam extends Query`：分页 Param 继承 `Query`，由框架携带 `pageNum` / `pageSize`；`companyCode` 字段标注 `@Schema(hidden = true)` 并由 Controller 注入，不暴露给前端。
- `ModifyTagDTO` 内嵌 `interface Modify {}` / `interface BatchModify {}`，实现分组校验——同一 DTO 在 `PUT /updatetag`（单条）和 `PUT /batchupdatetag`（批量）有不同必填规则。
- `PageInfo<CaseOrderRefundPageVO>`：由 PageHelper 框架填充，`@EnableResponseWrapper` 将其包装为标准响应结构，Controller 方法直接返回，无需手写包装层。
- 命名层：`CaseOrderRefundPageParam`（*Param，查询入参）、`ModifyTagDTO`（*DTO，命令/写操作入参）、`CaseOrderRefundPageVO` / `OrderRefundVO`（*VO，出参），三层分层见 esr-naming skill。
