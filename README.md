# Bandoo WebForge

Bandoo WebForge 是一个基于 Tauri v2 的桌面运行时，用来把 WebApp/PWA 封装成更接近原生体验的桌面应用。

产品方向很明确：原网页仍然是核心体验，Bandoo 在它外围补上桌面窗口、权限、脚本、自动化、快捷方式和本地集成能力。

## 当前状态

当前版本处于开发者内测阶段，主目标是验证 Windows、Linux 和 macOS 的核心闭环：

- 创建、编辑、删除和启动 WebApp。
- 使用本地 shell 承载自定义顶部栏，并在内容区域加载远程 WebView。
- 支持全局主题和单个 WebApp 的顶部栏外观覆盖。
- 支持窗口尺寸、位置、最大化状态保存与恢复。
- 使用 SQLite 保存 WebApp、设置、主题、自动化、脚本和运行日志。
- 通过受控 `window.__BANDOO__` Bridge 暴露页面、剪贴板、通知、Shell、文件系统和网络能力。
- 高风险能力默认关闭，并在 UI 中展示风险说明和最近调用日志。
- 支持自动化触发、步骤编辑、录制、选择器采集和执行日志。
- 支持用户脚本手动运行、页面加载、URL 变化和快捷键运行。
- 支持 Linux `.desktop` 入口和 Windows 快捷方式。
- 支持 macOS `.app` 快捷入口和 LaunchAgent 自启动入口。
- GitHub Actions 可以完成 Windows NSIS、Linux DEB 和 macOS DMG 内测产物构建。

## 安装内测版

推荐使用最新 beta Release：

- Windows：下载 `Bandoo.WebForge_0.1.0_x64-setup.exe`
- Linux：下载 `Bandoo.WebForge_0.1.0_amd64.deb`
- macOS：下载 `.dmg` 安装镜像

发布页：

- https://github.com/bandoo-2001/bandoo-web-forge/releases

详细验收流程见 [docs/BETA_ACCEPTANCE.md](docs/BETA_ACCEPTANCE.md)。

## 本地开发

安装依赖：

```bash
npm install
```

启动前端预览：

```bash
npm run dev -- --host 127.0.0.1
```

启动 Tauri 桌面端：

```bash
npm run tauri:dev
```

如果当前 shell 找不到 `cargo`，先加载 Rust 环境：

```bash
. "$HOME/.cargo/env"
```

## 验证命令

```bash
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Windows 本机如果 `cargo` 不在 `PATH`，可以先加入：

```powershell
$env:Path = "$HOME\.cargo\bin;$env:Path"
```

## 打包命令

Windows NSIS：

```bash
npm run tauri:build:windows
```

Linux DEB：

```bash
npm run tauri:build:linux
```

macOS DMG：

```bash
npm run tauri:build:macos
```

当前 Linux Release 只构建 `.deb`，AppImage/RPM 会在后续稳定后再恢复。当前 macOS Release 先构建 Intel DMG，签名、公证和 Universal Binary 会在后续处理。

## 项目结构

- `src/`：Vue 3 前端、Pinia stores、页面和样式。
- `src/types/`：前端共享 TypeScript 类型。
- `src-tauri/src/`：Rust 原生层、存储、运行时、Bridge、平台集成。
- `src-tauri/capabilities/`：Tauri v2 权限能力配置。
- `.github/workflows/ci.yml`：检查和 Release 打包流水线。
- `Bandoo_WebForge.md`：产品设计来源。

## 安全边界

- Shell、文件系统、网络权限默认关闭。
- 远程 WebView 只能通过受控 Bridge 请求高风险能力。
- Rust 侧按 WebApp ID、窗口 label 和权限开关做校验。
- 高风险调用会写入本地运行日志，方便内测定位问题。

## 已知限制

- Windows 安装包未签名，安装时可能出现安全提示。
- Linux 暂只发布 `.deb`。
- macOS 暂未签名/公证，首次打开可能需要用户手动确认。
- AppImage/RPM、自动更新、代码签名、公证、Universal Binary 和更完整的跨发行版验证属于下一轮发布工作。
