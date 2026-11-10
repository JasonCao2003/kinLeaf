# 枝间 Kinleaf 纸本谱册 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** 交付纸本谱册界面与可直接运行的 Windows x64 EXE。

**Architecture:** App 管理族谱选择与建谱入口。MemberEditor 保留数据和表单状态，分为世系导航与档案区域。共享样式定义纸色、松墨绿和键盘焦点。

**Tech Stack:** Vue、TypeScript、CSS、Vitest、Tauri。

**Spec:** [纸本谱册界面设计](../specs/2026-11-10-kinleaf-paper-ui-design.md)

## Global Constraints

- 保持现有 Tauri 命令和数据模型，使用本地样式和系统字体。
- 使用已有测试权限。构建 Windows x64 单文件 EXE。

## Review Focus

- 分区切换保留当前族员的输入，切谱后重新初始化界面。
- 搜索仍可定位折叠支系；窄窗口可滚动访问所有编辑区域。

### 任务 1 · 纸本谱册工作区

- [ ] 在 `src/App.test.ts` 添加分区切换测试，断言谱名草稿保留，运行 `npm run test -- --run` 确认新测试失败。
- [ ] 修改 `src/App.vue`，将切谱和建谱放入页首，初始状态显示独立引导。
- [ ] 修改 `src/components/MemberEditor.vue`，在右栏提供个人档案、支系登记、亲属关系和后代录入，使用本地视图状态控制分区显示。
- [ ] 调整 `src/components/LineageTree.vue`、`src/components/MemberSearch.vue`，新增 `src/paper.css` 并从 `src/main.ts` 加载。
- [ ] 运行 Vue 和 Rust 测试，确认通过；运行 `npm run tauri build -- --no-bundle --ci` 并对版本化 EXE 检查窗口启动。
- [ ] 经授权后考虑提交 `feat(ui): redesign Kinleaf as a paper genealogy workspace`。
