# Bandoo WebForge

增强型 WebApp/PWA 桌面运行时。目标是把任意网页转换为可增强、可自动化、可扩展的桌面应用。

## MVP 开发顺序

1. WebApp 管理：创建、保存、删除、独立窗口启动。
2. 运行时增强：托盘、全局快捷键、JS 注入。
3. 自动化系统：Trigger、Condition、Action。
4. 用户脚本：面向 JavaScript/TypeScript 的增强 API。

## 平台策略

项目按多平台运行时设计，当前实现以 Linux 为主。平台相关能力集中放在 Rust 侧的 `platform` 和 `runtime` 模块，后续桌面快捷方式、开机启动、托盘菜单、系统通知等功能优先实现 Linux，再为 Windows/macOS 增加对应分支。

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

## 验证命令

```bash
npm run build
. "$HOME/.cargo/env"
cd src-tauri
cargo fmt -- --check
cargo check
```

## 当前能力

- WebApp 创建、编辑、删除、启动、UserAgent、自定义图标字段、窗口状态恢复。
- WebApp 配置导入/导出 JSON。
- Linux `.desktop` 集成：应用菜单入口、桌面入口、autostart 自启动入口。
- 基础系统托盘：显示主窗口、退出。
- WebView 初始化脚本：注入 `window.__BANDOO__`，提供应用信息、权限、标题读取、路由监听和浏览器通知。
- 自动化步骤流：全局快捷键注册、绑定 WebApp 后执行、剪贴板读取、页面聚焦/输入/点击、通知和步骤级预检结果。
- 用户脚本：绑定 WebApp、启用/禁用、手动运行 JavaScript、权限预检和运行结果展示。
- Prompt 模板的最小数据模型和管理页面。

## 自动化 MVP 示例

1. 在 WebApp 管理里创建 ChatGPT WebApp，URL 使用 `https://chatgpt.com/`。
2. 打开该 WebApp 的权限：页面、剪贴板、通知。
3. 在自动化页面创建工作流：
   - 绑定 WebApp ID：填入 ChatGPT WebApp 的 ID。
   - 触发器：全局快捷键 `Ctrl+Alt+A`。
   - URL 条件：`chatgpt.com`。
   - 元素选择器：`#prompt-textarea, [data-testid="prompt-textarea"], textarea, [contenteditable="true"]`。
   - 输入模板：`{{clipboard}}`。
4. 复制一段文本到剪贴板，在 ChatGPT WebApp 窗口内按 `Ctrl+Alt+A`。
5. 自动化会读取剪贴板、聚焦输入框、填入文本，并在允许通知时提示完成。

如果快捷键保存失败，通常是格式无效或已有工作流占用了同一个快捷键。如果执行失败，先看自动化页的最近执行结果，再打开 WebApp 控制台查看 `[Bandoo automation]` 日志。

## 用户脚本 MVP 示例

1. 在 WebApp 管理里创建 ChatGPT WebApp，并开启页面和通知权限。
2. 在用户脚本页面创建脚本，绑定 ChatGPT WebApp ID。
3. 使用默认示例脚本，或填写：

```js
workflow.log('title:', bandoo.getTitle())
workflow.log('route:', bandoo.getRoute())
notification.send('Bandoo 用户脚本', `当前页面：${app.name}`)
```

4. 点击“运行”，Bandoo 会启动或聚焦绑定 WebApp，并把脚本派发到该 WebView。
5. 如果权限不足、脚本停用或 WebApp 未绑定，脚本不会执行，并会在用户脚本页显示原因。页面内异常会写入 WebApp 控制台的 `[Bandoo user script]` 日志。

## 常见问题

- `cargo` 找不到：执行 `. "$HOME/.cargo/env"` 后重试。
- Tauri 提示缺 Linux 依赖：确认已安装 WebKitGTK 4.1、librsvg、build-essential、libssl-dev、libayatana-appindicator3-dev。
- 桌面入口不可见：Linux 桌面环境可能需要重新加载应用菜单；桌面目录优先使用 `~/桌面`，不存在时使用 `~/Desktop`。
- 浏览器预览里桌面集成不可用：`.desktop`、托盘、独立 WebView 需要在 Tauri 运行时里使用。
