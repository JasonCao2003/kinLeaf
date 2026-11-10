# 枝间 Kinleaf 单机版核心功能 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** 交付可离线安装、建谱、修谱、查谱并备份恢复的 Windows 桌面应用。

**Architecture:** Vue 通过类型化的 Tauri 命令调用 Rust。Rust 负责事务、数据校验与文件发布，每份族谱使用一个独立 SQLite 文件。

**Tech Stack:** Tauri 2、Rust、Vue 3、TypeScript、Vite、npm、rusqlite（bundled、backup）、Vitest。

**Spec:** [首版核心功能设计](../specs/2026-11-10-famtree-core-design.md)

## Global Constraints

- Windows 首发，运行和安装支持断网。打包使用 NSIS 与 WebView2 离线安装模式。
- 每份族谱使用独立 SQLite 文件，存放在应用数据目录；以一个始祖为根。
- Access 迁移、分支合并、谱文排版、PDF 和其他出谱功能留在首版范围之外。
- 文本日期保持原始记法。恢复产生独立族谱标识。
- 锁定 Cargo 与 npm 依赖。记录实际验证的工具链版本。Windows 桌面打包使用 Windows 原生工具链。
- 测试运行与 Git 操作遵循用户授权。任务结束时报告未执行的检查。Git 提交建议仅供批准后的执行使用。

## Review Focus

- 建谱失败时，打开列表中没有半成品（任务 1）。
- 兼祧编辑失败时，个人档案与登记同时回滚（任务 2）。
- 通过另一条兼祧登记间接把自己挂到自己之下时拒绝修改（任务 2）。
- 搜索结果包含百分号和下划线时按字面匹配（任务 3）。
- SQLite 文件完整但存在孤立登记、谱系循环或错误版本时拒绝恢复（任务 4）。

## 文件与接口约定

- `src-tauri/migrations/001_init.sql` 定义表和约束。`src-tauri/src/store.rs` 负责本机文件发现、创建及连接。`model.rs` 定义 IPC 数据类型。
- `src-tauri/src/members.rs` 管理登记及亲属关系。`queries.rs` 提供浏览与搜索。`backup.rs` 管理快照和恢复。`commands.rs` 暴露命令。`lib.rs` 注册命令及权限范围。
- `src/api.ts` 集中封装 IPC。`src/App.vue` 管理当前族谱与选中登记。`src/components/` 内各组件分别管理族谱选择、世系树、档案编辑、搜索和备份交互。
- ID 在 IPC 中统一为 UUID 字符串。Rust DTO 用 serde camelCase。所有命令返回 Result<T, AppError>，错误包含稳定的 code 和可展示 message。
- 表采用 `family(id,name,root_entry_id)`、`person(id,name,spectrum_name,generation_name,birth_text,death_text,notes)`、`entry(id,person_id,parent_entry_id,rank,branch_notes)`、`parents(child_id,father_id,mother_id)`、`spouse(person_a_id,person_b_id)`。配偶对按 ID 规范排序并唯一。每次连接启用外键。rank 为可空正整数，空值排在末尾，平局按登记 ID 排序。
- 写命令按族谱 ID 解析应用数据目录内的目标。禁止从前端传入数据库连接路径。数据库访问与文件对话框隔离。原生对话框取消视为正常返回。

### 任务 1 · 从桌面新建并重新打开族谱

**文件**：`src-tauri/{Cargo.toml,tauri.conf.json,migrations/001_init.sql,src/{lib,commands,model,store}.rs,tests/store.rs}`、`src/{App.vue,api.ts,components/FamilyPicker.vue}`，以及前端入口、Vite 配置、清单和锁文件。

**接口**：`list_families() -> Vec<FamilySummary>`、`create_family(name: String, founder_name: String) -> FamilySummary`。FamilySummary 含 id、name、root_entry_id。测试直接注入临时数据目录。

- [ ] 建立 Rust 与 Vue 构建入口，记录 Windows 本机开发依赖。
- [ ] 添加 `create_reopen_and_ignore_failed_creation`。断言创建后用新 Store 能找到同一 id 和根登记；空白始祖名失败且列表不变；未发布的临时文件不会被枚举。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test store`，确认创建行为尚未实现。
- [ ] 在同目录暂存 SQLite 文件，事务创建谱、个人和根登记，关闭连接后发布。接通族谱选择器和创建表单。
- [ ] 重跑 store 测试，预期全部通过。运行 `npm run build`，预期构建成功。用 `npm run tauri dev` 手动验证重启后可重新打开。
- [ ] 经授权后考虑提交 `feat(core): create and reopen local family trees`。

### 任务 2 · 编辑族员、兼祧与亲属关系

**文件**：新增 `src-tauri/src/members.rs`、`src-tauri/tests/members.rs`、`src/components/MemberEditor.vue`，修改命令注册、DTO 和前端 API。

**接口**：以下命令首参均为 `family_id: String`。`add_child(parent_entry_id: String, person: PersonChoice, fields: EntryFields) -> String`；`update_entry(entry_id: String, parent_entry_id: Option<String>, fields: EntryFields) -> ()`；`update_person(person_id: String, fields: PersonFields) -> ()`；`set_parents(person_id: String, father_id: Option<String>, mother_id: Option<String>) -> ()`；`set_spouse(person_id: String, other_id: String, linked: bool) -> ()`；`delete_entry(entry_id: String) -> ()`；`find_people(query: String) -> Vec<PersonSummary>`；`create_person(fields: PersonFields) -> String`，用于创建尚未登记的配偶或父母。PersonChoice 为 New(PersonFields) 或 Existing(String)。EntryFields 含 rank、branch_notes。PersonFields 对应 person 文本列；PersonSummary 含 id、name、spectrum_name。

- [ ] 添加 `shared_profile_and_branch_fields`、`failed_add_leaves_no_person`、`reject_entry_cycles_and_ancestor_with_same_person`、`parents_and_spouse_rules`。断言兼祧共享姓名且各有排行、失败新增前后档案数相同、跨支系迁移后的路径无同人祖先、失败迁移保留原上级、拒绝亲生循环及自配偶、两位配偶可并存、删除叶登记保留亲属关系。覆盖始祖和有子节点的删除拒绝。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test members`，确认上述规则待实现。
- [ ] 实现事务写入。移动时检查目标祖先链与待移动子树，包括跨登记的同人冲突。生父母使用独立关系图判断循环；关系的两端必须属于本谱。
- [ ] 实现编辑表单与现有档案选择。共享档案修改前提示影响其他登记。详情的读接口和世系联动由任务 3 接通。
- [ ] 重跑 members 测试，预期全部通过。运行 `npm run build`，预期构建成功。
- [ ] 经授权后考虑提交 `feat(members): edit lineage registrations and relatives`。

### 任务 3 · 浏览世系并定位搜索结果

**文件**：新增 `src-tauri/src/queries.rs`、`src-tauri/tests/queries.rs`、`src/components/{LineageTree,MemberSearch}.vue` 和 `src/App.test.ts`。更新命令、DTO、API 和 App。

**接口**：首参均为 `family_id: String`。`list_children(parent_entry_id: String) -> Vec<EntrySummary>`、`get_entry(entry_id: String) -> EntryDetail`、`search_entries(query: String) -> Vec<SearchHit>`。EntrySummary 含 entry_id、person_id、name、rank、has_children。SearchHit 含 entry、generation、path；path 是从始祖到命中的 EntrySummary 序列。EntryDetail 含 hit、PersonFields、EntryFields、已知父母与配偶 PersonSummary、同一人的其他 SearchHit。命中按登记返回。

- [ ] 添加 `search_paths_preserve_distinct_registrations` 与 `search_treats_wildcards_literally`。断言同名两人和同人双登记分别命中，path 首项为始祖，末项为目标，generation 等于 path 长度；确认按谱名也能找到、空查询清空结果、百分号与下划线按字面匹配、同排行重复读取顺序一致。
- [ ] 添加 App 测试，模拟点击搜索命中，断言展开返回路径并显示目标详情；切换族谱后旧请求的结果不可覆盖当前状态。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test queries` 和 `npm run test -- --run`，确认失败指向尚未实现的读取和定位。
- [ ] 实现 SQLite 读取与稳定排序。通过查询返回的路径逐层展开并选中目标。接通详情与编辑表单，保存后刷新相关树节点和详情。
- [ ] 重跑查询测试和 Vitest，预期全部通过。运行 `npm run build`，预期构建成功。
- [ ] 经授权后考虑提交 `feat(lineage): browse and locate family members`。

### 任务 4 · 备份恢复及 Windows 离线交付

**文件**：新增 `src-tauri/src/backup.rs`、`src-tauri/tests/backup.rs`、`src/components/BackupActions.vue`、`docs/verification/windows-offline.md`；修改命令注册、API、App 和打包配置。

**接口**：`export_backup(family_id: String) -> Option<()>`、`restore_backup() -> Option<FamilySummary>`。命令通过原生对话框选路径。后端文件函数 `export_to(store: &Store, family_id: &str, destination: &Path) -> Result<(), AppError>`、`restore_from(store: &Store, source: &Path) -> Result<FamilySummary, AppError>` 可直接测试。

- [ ] 添加 `restore_creates_editable_independent_family`、`restore_rejects_invalid_files_without_changing_catalog`、`export_refuses_existing_destination`。断言 WAL 中已提交的数据可恢复，恢复 ID 不同且编辑副本不影响原谱；损坏文件、错误版本、孤立登记与循环恢复均失败且目录清单一致；既有目标内容保留。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test backup`，确认快照与恢复待实现。
- [ ] 使用 SQLite backup API 输出一致快照。恢复时以只读方式读取来源并暂存到应用目录。检查 SQLite application_id、user_version、完整性、外键及任务 2 的关系规则，校验根唯一且全部登记可达，替换标识后关闭连接并发布。失败时清理本次暂存文件。
- [ ] 接通备份与恢复按钮。恢复后打开副本，取消时保留当前谱。导出仅创建新文件。
- [ ] 经授权后运行 `cargo test --manifest-path src-tauri/Cargo.toml`、`npm run test -- --run`、`npm run build`、`npm run tauri build -- --bundles nsis`。预期测试通过并产出 NSIS 安装包。在断网 Windows 环境按验证文档执行安装、建谱、修谱、搜索、重启、跨环境恢复以及损坏备份拒绝场景。
- [ ] 经授权后考虑提交 `feat(backup): restore local snapshots and package offline installer`。
