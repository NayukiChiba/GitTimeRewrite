# Git Time Rewrite

Rust + Vue3 + Tauri 的 Git 历史时间编辑桌面工具。

## 目录结构

- frontend：Vue3 前端（页面、API 封装、前端测试）
- backend：Tauri + Rust 后端（Git 操作命令、后端测试）

## 核心能力

- 选择本地 Git 仓库目录
- 查看仓库提交历史（按时间正序）
- 逐条编辑提交 message / 作者 / 时间
- 批量按日期连续 + 时间随机重建时间线
- 支持跨天时间窗口（如 08:00 到次日 01:00）
- 批量设置作者信息
- 前后端通过 Tauri invoke API 交互

## 开发启动

```bash
npm --prefix frontend install
npm --prefix frontend run tauri:dev
```

## 打包

```bash
npm --prefix frontend run tauri:build
```

## 测试

```bash
npm --prefix frontend run test:unit -- --run
npm --prefix frontend run type-check
cargo test --manifest-path backend/Cargo.toml
```
