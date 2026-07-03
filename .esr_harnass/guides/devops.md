# Guide: docs/devops.md

本文件说明如何生成业务项目的 `docs/devops.md`。

## 归属准则：devops vs infra

`docs/devops.md` 只记录**流程动作**——部署新版本时会随之改变、或需要人工执行的操作步骤、命令与策略。

与 `docs/infra.md` 的分界：如果某项信息在部署新版本后不会改变（例如服务拓扑、固定端口、常驻进程清单、JVM 静态参数、数据目录路径），它属于静态事实，归 `infra.md`，不属于 `devops.md`。两份文档内容不重叠：静态事实只写在 `infra.md`，流程动作只写在 `devops.md`。

## Build Verification 写法

写项目自身声明的快速校验命令（编译/类型检查），以及可选的打包校验命令和触发条件。builder-agent 以此节为唯一命令来源，不自行推断。

- **快速校验**（必写）：列出能在秒级完成的编译/类型检查命令，无需打包；这是 builder-agent 完成任务后必跑的最低一档。
- **打包校验**（可选）：列出完整打包命令，并明确标注"影响打包的文件范围"（如 `pom.xml`、`Dockerfile`、资源目录等）；agent 只在改动命中该范围时才执行。
- 禁止写无法实际执行、或仅供示意的假命令。

## 部署流程写法

写实际执行的部署步骤和命令。不写静态拓扑事实（服务 IP、节点角色、机器数量等，那些归 `infra.md`）。

通常包括：打包策略、产物分发方式、服务器安装命令、systemd 服务注册与启停命令。每条命令应可直接复制执行。

## 运维命令写法

写日常运维操作，例如：重启服务、滚动升级、健康检查命令。不写固定端口或机器 IP（那些是静态事实，归 `infra.md`）。

建议按场景分小节组织，例如"快速状态检查"、"服务管理"、"日志查看"，方便 agent 和人快速定位。

## 故障排查写法

写排查步骤、日志查看命令、常见问题处理方式。不写静态存储路径（日志目录位置归 `infra.md`）。

## 分支策略写法

写本项目实际采用的分支命名规范、合并规则、PR/MR 创建要求。

## Config 块写法

Config 块**必须保留在 `devops.md`**，不迁移到 `infra.md`。agent 从此块读取框架级配置参数：

- `vcs_platform`：取值 `gh` / `glab` / `gh+glab` / `none`，agent 据此选择 VCS CLI 工具。
- `review_policy`：取值 `auto` / `cautious` / `human_required`，agent 据此决定 review 阶段的自动化程度。
- 如有其他框架级配置键，同样写在 Config 块。

## 禁止事项

- 不写静态环境事实（拓扑、组件配置、存储位置、常驻 cron/watchdog、JVM 参数等）——这些归 `infra.md`。
- 不写 builder、qa、agent 行为规则。
- 不写目标、价值、原则等无信息量说明。

## 示例骨架

以下展示 `docs/devops.md` 的典型结构：

```markdown
# DevOps

## Config

vcs_platform: gh
review_policy: auto

## Build Verification

快速校验（必跑，编译 + 类型检查，无需打包）：

    mvn -q compile

打包校验（命中以下文件范围时执行）：

    mvn -q package -DskipTests

影响打包的文件范围：`pom.xml`、`src/main/resources/`

## 部署方式

- 打包：`bash deploy/build.sh --env prod`
- 服务器安装：`MYAPP_ENV=prod bash install.sh`
- systemd 服务注册后由 `systemctl start my-service` 启动，开机自启

## 常用运维命令

查看服务状态：
    systemctl status my-service --no-pager

实时跟踪日志：
    tail -f /var/log/my-service/app.log

重启服务：
    systemctl restart my-service

## 故障排查

查看最近错误：
    grep "ERROR" /var/log/my-service/app.log | tail -20

检查 checkpoint 是否正常：
    grep "Completed checkpoint" /var/log/my-service/app.log | tail -3

## 分支策略

功能分支 `feature/ITER-NNN`，基于 `main` 创建，通过 PR 合并，合并前需 CI 通过。
```
