# Bandoo WebForge

增强型 WebApp/PWA 桌面运行时。目标是把任意网页转换为可增强、可自动化、可扩展的桌面应用。

## MVP 开发顺序

1. WebApp 管理：创建、保存、删除、独立窗口启动。
2. 运行时增强：托盘、全局快捷键、JS 注入。
3. 自动化系统：Trigger、Condition、Action。
4. 用户脚本：面向 JavaScript/TypeScript 的增强 API。

## 本地开发

```bash
npm install
npm run dev
npm run tauri:dev
```

当前机器已安装 Tauri 需要的 Ubuntu 系统依赖和 Rust stable。新 shell 如果还找不到 `cargo`，先执行：

```bash
. "$HOME/.cargo/env"
```
