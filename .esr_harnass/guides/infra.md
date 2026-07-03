# Guide: docs/infra.md

本文件说明如何生成业务项目的 `docs/infra.md`。

## 归属准则：infra vs devops

`docs/infra.md` 只记录**静态环境事实**——部署新版本后不会改变的信息，例如服务拓扑、固定端口、JVM 静态参数、数据目录。

与 `docs/devops.md` 的分界：如果某项信息会随部署更新（例如操作步骤、部署命令、滚动升级策略），它属于流程动作，归 `devops.md`，不属于 `infra.md`。两份文档内容不重叠：静态事实只写在 `infra.md`，流程动作只写在 `devops.md`。

## 服务器拓扑写法

列出各节点角色、机器数量、所在网络区。不写如何部署到这些节点（部署步骤归 `devops.md`）。

例如：正式环境 Flink 节点 1 台（10.9.0.112，16 核 / 32 GB，正式网段），Kafka 3 节点（10.9.0.113-115）。

## 组件配置写法

写各组件的静态参数，例如端口、连接数、内存分配、关键配置项。不写如何修改这些配置（修改操作归 `devops.md`）。

例如：Trino 端口 8080，`query.max-memory=16GB`，`task.concurrency=8`。

## 存储位置写法

写数据目录、日志目录、挂载点、S3 Bucket 名称和路径。不写如何清理或迁移这些存储（操作步骤归 `devops.md`）。

例如：日志目录 `/var/log/myapp/`，checkpoint 目录 `file:///var/log/myapp/checkpoints`，数据桶 `s3a://my-bucket/`。

## 常驻 cron / watchdog 写法

列出常驻进程或定时任务的名称、触发周期、宿主机。不写触发逻辑或维护操作（那些归 `devops.md`）。

例如：`adms-iceberg-maintenance` cron，每天 03:00 触发，宿主 10.9.0.112；`adms-watchdog` 每 10 分钟触发，同宿主。

## 访问方式写法

写固定的内网域名、IP、端口。不写访问步骤或 VPN 使用说明（操作步骤归 `devops.md`）。

例如：Trino JDBC `jdbc:trino://10.9.0.111:8080/iceberg/adms_hourly`，Kafka broker `10.9.0.113:9092,10.9.0.114:9092`。

## 部署参数写法

写约定的静态值，例如 JVM 内存（`-Xmx`）、网络缓冲大小（socket buffer size）、并行度上限。不写如何调优（调优建议归 `devops.md` 或迭代 tech-design.md）。

例如：ingest 进程 JVM `-Xmx12288m`，kafka-push 进程 `-Xmx12288m`，`system.parallelism=4`（生产固定，不调整）。

## 禁止事项

- 不写启动命令或操作步骤——这些归 `devops.md`。
- 不写如何修改配置、如何清理存储、如何维护 cron。
- 不写 builder、qa、agent 行为规则。
- 不写目标、价值、原则等无信息量说明。

## 示例骨架

以下展示 `docs/infra.md` 的典型结构：

```markdown
# Infra

## 服务器拓扑

| 服务 | IP | CPU | 内存 | 备注 |
|------|-----|-----|------|------|
| Flink (ingest + kafka-push) | 10.9.0.112 | 16 核 | 32 GB | 正式网段 |
| Trino + Hive Metastore | 10.9.0.111 | 8 核 | 32 GB | |
| Kafka (×3) | 10.9.0.113-115 | 4 核 | 16 GB | 三节点集群 |

## 访问方式

- Kafka broker：`10.9.0.113:9092,10.9.0.114:9092,10.9.0.115:9092`
- Trino JDBC：`jdbc:trino://10.9.0.111:8080/iceberg/mydb`
- MinIO Endpoint：`http://minio.example.com:9000`

## 组件配置

### Trino

- 端口：8080
- `query.max-memory=16GB`
- `task.concurrency=8`

### Flink (ingest)

- `system.parallelism=4`（生产固定）
- 网络缓冲：`taskmanager.memory.network.min=max=128mb`

## 存储位置

- 日志：`/var/log/myapp/*.log`（logrotate 每日轮转保留 7 天）
- Checkpoint：`file:///var/log/myapp/checkpoints/{ingest,kafka-push}/`
- S3 数据桶：`s3a://my-bucket/<table>`（metadata/ + data/）

## 常驻 cron / watchdog

| 名称 | 触发周期 | 宿主机 |
|------|---------|--------|
| `myapp-maintenance` cron | 每天 03:00 | 10.9.0.112 |
| `myapp-watchdog` cron | 每 10 分钟 | 10.9.0.112 |

## 部署参数

- ingest JVM：`-Xmx12288m`
- kafka-push JVM：`-Xmx12288m`
- 生产并行度：`system.parallelism=4`（不调整）
```
