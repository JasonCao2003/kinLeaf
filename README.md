# 枝间 Kinleaf

枝间是一款 Windows 单机族谱软件。以纸本谱册为界面灵感，用来整理世系、维护族员档案和记录家人的故事。

## 下载与使用

1. 从 [GitHub Releases](https://github.com/JasonCao2003/kinleaf/releases/latest) 下载 Windows x64 单文件 EXE。
2. 双击打开。运行环境需要 Microsoft Edge WebView2 Runtime。
3. 点击「新建族谱」，填写谱名和始祖姓名。已有族谱可从页首切换。
4. 在左侧展开世系或按姓名、谱名搜索。在右侧编辑个人档案、支系登记和亲属关系。

## 已发布功能

- 建立和切换本机族谱。
- 登记后代、维护生父母和配偶关系。
- 支持查看兼祧登记，同一人的多个支系登记共享个人档案。
- 世系树以虚线标明层级，支持整支折叠和搜索定位。
- 通过「系统工具」备份当前族谱，并恢复为独立副本。

族谱数据存放在本机的 `%APPDATA%\com.kinleaf.desktop\families`，每谱一个 SQLite 文件。EXE 可以单独移动，数据仍保存在该用户目录。

## 开发

技术栈为 Tauri、Rust、Vue、TypeScript 和 SQLite。Windows 桌面构建使用原生 Rust MSVC 工具链、Microsoft C++ Build Tools、Node.js 与 npm。

```powershell
npm ci
npm run tauri dev
npm run test -- --run
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build -- --no-bundle --ci
```

单文件构建结果位于 `src-tauri/target/release/kinleaf.exe`。后续开发顺序见[路线文档](docs/superpowers/plans/2026-11-10-kinleaf-roadmap.md)。
