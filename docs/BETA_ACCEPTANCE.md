# Bandoo WebForge Beta Acceptance

这份文档用于内测前的手工验收。目标不是覆盖所有边角，而是确认开发者可以安装、创建 WebApp、自定义窗口、运行基础自动化和脚本，并能看见权限与错误。

## 下载与安装

从 GitHub Releases 下载最新 beta：

- Windows：`Bandoo.WebForge_0.1.0_x64-setup.exe`
- Linux：`Bandoo.WebForge_0.1.0_amd64.deb`
- macOS：`.dmg` 安装镜像

Linux 安装示例：

```bash
sudo apt install ./Bandoo.WebForge_0.1.0_amd64.deb
```

Windows 安装包暂未签名，安装时出现安全提示属于当前已知限制。
macOS 安装包暂未签名和公证，首次打开时出现 Gatekeeper 提示属于当前已知限制。

## 核心验收路径

1. 启动 Bandoo WebForge。
2. 创建一个 WebApp，例如：
   - 名称：`ChatGPT`
   - URL：`https://chatgpt.com/`
3. 保存并启动 WebApp。
4. 确认打开的是独立 Bandoo shell 窗口，而不是外部浏览器。
5. 修改该 WebApp 的顶部栏外观：
   - 背景色
   - 文字色
   - 顶部栏高度
   - 圆角
   - 按钮位置
6. 再次启动 WebApp，确认顶部栏样式生效，远程网页内容区域没有被遮挡。
7. 缩放窗口，确认内容 WebView 跟随尺寸变化。
8. 关闭并重新打开，确认窗口位置、大小和最大化状态能恢复。

## 权限验收

默认状态下，高风险能力应关闭：

- Shell
- 文件系统
- 网络

验收步骤：

1. 在 WebApp 设置中确认高风险权限默认关闭。
2. 打开 Shell 权限，确认 UI 显示风险说明。
3. 触发一次 Bridge 调用，确认运行日志出现记录。
4. 关闭对应权限，再触发同类调用，确认 Rust 侧拒绝请求并写入失败日志。

远程页面不能直接调用管理命令，只能通过受控 `window.__BANDOO__` Bridge 请求能力。

## 自动化验收

1. 创建一个绑定到 ChatGPT WebApp 的自动化。
2. 设置触发器为快捷键，例如 `Ctrl+Alt+A`。
3. 添加步骤：
   - 等待页面
   - 聚焦输入框
   - 输入剪贴板内容
   - 发送通知
4. 复制一段文本到剪贴板。
5. 在 WebApp 窗口中按快捷键。
6. 确认步骤逐条完成，执行日志包含每步状态、耗时和错误信息。

选择器采集验收：

1. 在自动化步骤编辑器中进入选择模式。
2. 到目标 WebView 中点击输入元素。
3. 确认 selector 回填到当前步骤。

录制模式验收：

1. 开启录制。
2. 在目标页面中点击、输入、等待。
3. 停止录制。
4. 确认生成对应步骤，并可再次执行。

## 用户脚本验收

1. 创建用户脚本并绑定到 ChatGPT WebApp。
2. 设置语言为 JavaScript 或 TypeScript。
3. 配置 match pattern，例如 `https://chatgpt.com/*`。
4. 手动运行一次，确认日志面板出现输出。
5. 设置 `runAt` 为页面加载或 URL 变化。
6. 重新打开或切换页面，确认脚本自动运行。
7. 故意写入一段错误脚本，确认错误能在 UI 中展示。

## 桌面集成验收

Linux：

1. 安装 `.desktop` 入口。
2. 确认应用菜单或桌面入口可启动 WebApp。
3. 安装自启动入口。
4. 删除 WebApp，确认关联入口被清理。

Windows：

1. 创建开始菜单快捷方式。
2. 创建桌面快捷方式。
3. 创建启动目录快捷方式。
4. 删除 WebApp，确认关联快捷方式被清理。

macOS：

1. 创建应用入口，确认 `~/Applications` 下出现 WebApp `.app` wrapper。
2. 创建桌面入口，确认桌面出现 WebApp `.app` wrapper。
3. 创建自启动入口，确认 `~/Library/LaunchAgents` 下出现对应 `.plist`。
4. 删除 WebApp，确认关联 `.app` 和 LaunchAgent 被清理。

## 发布验收

发布前至少确认：

- `npm test` 通过。
- `npm run build` 通过。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` 通过。
- `cargo check --manifest-path src-tauri/Cargo.toml` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml` 通过。
- GitHub Actions Windows checks 通过。
- GitHub Actions Ubuntu checks 通过。
- Release 上传 Windows `.exe`。
- Release 上传 Linux `.deb`。
- Release 上传 macOS `.dmg`。

## 当前已知限制

- Windows 安装包未签名。
- Linux 仅发布 `.deb`。
- macOS 安装包未签名、未公证，且暂不发布 Universal Binary。
- AppImage/RPM、自动更新、代码签名和 macOS 公证将在后续版本处理。
