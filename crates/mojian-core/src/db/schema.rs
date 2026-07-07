//! 中央 DB schema：版本常量 + v1 建表 SQL。
//!
//! `V1_SCHEMA_SQL` 逐字对齐 `docs/tech-design/storage.md#五、DB 表设计` 的 12 表 DDL。
//! 这里是 DDL 的单一事实源；建表由迁移运行器在单事务内以 `execute_batch` 执行。

/// 当前 schema 版本。迁移以 `schema_meta.schema_version` 为键，不用 PRAGMA user_version。
pub const SCHEMA_VERSION: i64 = 1;

/// v1 建表脚本：storage.md「五」逐字 12 表（project / project_state / reference_book /
/// volume / batch / chapter / artifact_ref / bible_version / void_record / stat / config /
/// schema_meta）。列名 snake_case，SQLite 类型。
pub const V1_SCHEMA_SQL: &str = r#"
CREATE TABLE project (
  project_id    TEXT PRIMARY KEY,          -- UUID
  name          TEXT NOT NULL,
  path          TEXT NOT NULL,             -- 项目目录绝对路径
  spec_version  TEXT,                      -- 已部署的 SPEC 版本
  spec_hash     TEXT,                      -- 部署缓存校验用
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE TABLE project_state (               -- 1:1 project
  project_id       TEXT PRIMARY KEY REFERENCES project(project_id),
  sop_phase        TEXT NOT NULL,          -- style_sampling … bible_building … writing
  current_volume   TEXT REFERENCES volume(id),
  cursors          TEXT,                   -- JSON：跨步骤游标（如抽取块游标聚合视图）
  updated_at       TEXT NOT NULL
);

CREATE TABLE reference_book (              -- SOP① 参考小说
  id             TEXT PRIMARY KEY,
  project_id     TEXT NOT NULL REFERENCES project(project_id),
  title          TEXT NOT NULL,
  extract_status TEXT NOT NULL,            -- pending | extracting | extracted
  block_cursor   INTEGER NOT NULL DEFAULT 0,  -- 当前 ~5万字块索引
  updated_at     TEXT NOT NULL
);

CREATE TABLE volume (                      -- 卷 / Arc
  id                TEXT PRIMARY KEY,       -- ARC-xxx
  project_id        TEXT NOT NULL REFERENCES project(project_id),
  seq               INTEGER NOT NULL,
  name              TEXT,
  arc_phase         TEXT NOT NULL,          -- arc_planning | arc_plan_review | writing | arc_done
  chapters_total    INTEGER NOT NULL DEFAULT 0,
  chapters_approved INTEGER NOT NULL DEFAULT 0,
  deviation         INTEGER NOT NULL DEFAULT 0,  -- 实际章数 - 大纲预期
  updated_at        TEXT NOT NULL
);

CREATE TABLE batch (                       -- 调度单位（每批 3-5 章）
  id          TEXT PRIMARY KEY,
  project_id  TEXT NOT NULL REFERENCES project(project_id),
  volume_id   TEXT NOT NULL REFERENCES volume(id),
  status      TEXT NOT NULL,
  created_at  TEXT NOT NULL
);

CREATE TABLE chapter (                     -- 最细状态机载体
  id             TEXT PRIMARY KEY,          -- CH-xxx
  project_id     TEXT NOT NULL REFERENCES project(project_id),
  volume_id      TEXT NOT NULL REFERENCES volume(id),
  batch_id       TEXT REFERENCES batch(id), -- 未入批为 NULL
  seq            INTEGER NOT NULL,
  status         TEXT NOT NULL,             -- planned | skeleton_drafting | skeleton_review | prose_drafting | prose_review | approved | void
  verify_flag    TEXT,                      -- clean | suspect
  skeleton_path  TEXT,  skeleton_hash TEXT,
  prose_path     TEXT,  prose_hash    TEXT,
  updated_at     TEXT NOT NULL
);

CREATE TABLE artifact_ref (                -- DB↔SSOT 的桥
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id   TEXT NOT NULL REFERENCES project(project_id),
  path         TEXT NOT NULL,
  kind         TEXT NOT NULL,              -- input | process | result
  content_hash TEXT NOT NULL,              -- 认变更、驱动过期检测
  version      INTEGER NOT NULL DEFAULT 1,
  updated_at   TEXT NOT NULL
);

CREATE TABLE bible_version (               -- 圣经版本化
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id  TEXT NOT NULL REFERENCES project(project_id),
  version     INTEGER NOT NULL,
  reason      TEXT,
  trigger     TEXT NOT NULL,               -- void | human
  created_at  TEXT NOT NULL
);

CREATE TABLE void_record (                 -- 作废记录
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id     TEXT NOT NULL REFERENCES project(project_id),
  chapter_id     TEXT NOT NULL REFERENCES chapter(id),
  reason         TEXT,
  affected_scope TEXT,                      -- JSON：受影响章节 id 列表
  created_at     TEXT NOT NULL
);

CREATE TABLE stat (                        -- 节奏统计
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id  TEXT NOT NULL REFERENCES project(project_id),
  scope       TEXT NOT NULL,               -- book:{id} | chapter:{id}
  metric      TEXT NOT NULL,               -- dialogue_ratio | hanzi | hook_density | …
  value       REAL NOT NULL,
  updated_at  TEXT NOT NULL
);

CREATE TABLE config (                      -- 项目配置覆盖项（默认在 defaults.toml）
  project_id  TEXT NOT NULL REFERENCES project(project_id),
  key         TEXT NOT NULL,
  value       TEXT NOT NULL,
  PRIMARY KEY (project_id, key)
);

CREATE TABLE schema_meta (                 -- 迁移用
  schema_version INTEGER NOT NULL
);
"#;
