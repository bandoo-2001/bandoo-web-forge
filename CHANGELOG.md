# Changelog

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
