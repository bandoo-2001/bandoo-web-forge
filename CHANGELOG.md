# Changelog

## Unreleased

### Added

- 实验性 macOS 支持：CI checks、DMG 打包脚本和 Release 资产上传。
- macOS 桌面集成：`~/Applications` / 桌面 `.app` wrapper，以及 `~/Library/LaunchAgents` 自启动入口。
- 运行时平台信息增加 macOS 与桌面集成支持状态。

### Known Limitations

- macOS 产物暂未签名、公证，也暂不发布 Universal Binary。
- macOS 暂不启用 Tauri builder 级透明窗口，透明和圆角优先走内容层退化。

## v0.1.0-beta.3

内测文档与发布卫生收口版本。

### Added

- 重写 README，恢复可读的中文项目入口、开发命令、打包命令和已知限制。
- 新增 `docs/BETA_ACCEPTANCE.md`，覆盖安装、窗口、权限、自动化、脚本和桌面集成验收。
- 新增 changelog，记录内测能力和已知限制。

### Changed

- Beta tag 自动标记为 GitHub prerelease。
- Release workflow 自动生成 GitHub release notes。

## v0.1.0-beta.2

内测发布入口。该版本对应提交 `d092d4d`。

### Added

- SQLite 本地存储、schema version、旧 JSON 首次导入和备份。
- 自定义 WebApp shell 窗口、顶部栏和远程内容 WebView。
- 全局主题、单 App 外观覆盖、主题导入导出和实时预览。
- 受控 `window.__BANDOO__` Bridge，以及 Shell、文件系统、网络权限校验。
- 自动化步骤编辑、选择器采集、录制模式和运行日志。
- 用户脚本 match patterns、runAt、权限声明、TypeScript 保存时转译和日志面板。
- Linux `.desktop` 入口和 Windows 快捷方式集成。
- GitHub Actions Windows NSIS 与 Linux DEB Release 打包。

### Changed

- Linux Release 产物收窄为稳定的 `.deb`。
- Windows Release 产物使用 NSIS 安装包。

### Known Limitations

- macOS 暂不支持。
- Windows 安装包暂未签名。
- Linux 暂不发布 AppImage/RPM。
- 自动更新和代码签名不在本轮内测范围内。
