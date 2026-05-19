# Bandoo WebApp Runtime（暂定）
## 基于 Tauri 的增强型 PWA/WebApp 桌面运行时设计文档（MVP）

---

# 1. 项目定位

Bandoo WebApp Runtime 是一个：

```text
增强型 WebApp/PWA 桌面运行时
```

目标：

```text
将任意网页转换为可增强、可自动化、可扩展的桌面应用
```

核心理念：

```text
不是重新实现网页
而是在原网页基础上进行桌面级增强
```

---

# 2. 产品目标

解决当前 Linux/Ubuntu 下：

- ChatGPT 无官方客户端
- 浏览器 PWA 能力弱
- 浏览器插件权限有限
- 自动化能力不足
- 缺乏统一 WebApp 管理

问题。

---

# 3. 核心能力

## 3.1 WebApp/PWA 创建

支持：

- 输入 URL 创建桌面应用
- 独立窗口运行
- 自定义图标
- 自定义窗口大小
- 自定义 UserAgent
- 开机启动
- 托盘运行

示例：

```text
https://chatgpt.com
↓
生成 ChatGPT 桌面应用
```

---

## 3.2 WebApp 增强能力

### 支持：

- 全局快捷键
- 页面脚本注入
- 自定义菜单
- 路由匹配
- 自动化任务
- 用户脚本
- 剪贴板增强
- AI Prompt 模板
- 系统通知

---

## 3.3 自动化系统

两类用户：

| 用户类型 | 创建方式 |
|---|---|
| 普通用户 | 图形化拖拽/步骤配置 |
| 开发者 | JavaScript/TypeScript脚本 |

统一执行引擎。

---

# 4. 技术栈

## 4.1 桌面端

| 技术 | 用途 |
|---|---|
| Tauri v2 | 桌面运行时 |
| Rust | 系统能力层 |
| WebView | 网页容器 |

---

## 4.2 前端

| 技术 | 用途 |
|---|---|
| Vue3 | UI框架 |
| TypeScript | 类型系统 |
| SCSS | 样式扩展 |
| TailwindCSS | 原子化UI |
| Pinia | 状态管理 |
| Vue Router | 路由 |
| Vite | 构建工具 |

---

## 4.3 数据存储

| 技术 | 用途 |
|---|---|
| SQLite | 本地数据库 |
| JSON | 工作流配置 |

---

# 5. 系统架构

```text
┌──────────────────────┐
│ Vue Frontend         │
│                      │
│ App管理              │
│ 自动化编辑器          │
│ 用户脚本编辑器        │
│ 设置中心              │
└─────────┬────────────┘
          │ invoke/event
┌─────────▼────────────┐
│ Tauri Core (Rust)    │
│                      │
│ Window管理           │
│ Shortcut管理         │
│ Tray管理             │
│ Menu管理             │
│ 权限控制              │
│ 文件系统              │
│ 自动化执行引擎        │
└─────────┬────────────┘
          │
┌─────────▼────────────┐
│ WebView Runtime      │
│                      │
│ 页面注入              │
│ DOM操作               │
│ Route监听             │
│ JS Bridge             │
└──────────────────────┘
```

---

# 6. 核心模块设计

## 6.1 WebApp 管理模块

### 功能

- 创建应用
- 编辑应用
- 删除应用
- 导入导出
- 启动应用
- 窗口恢复

### 数据结构

```ts
interface WebApp {
  id: string
  name: string
  icon?: string
  url: string

  windowConfig: {
    width: number
    height: number
    maximized?: boolean
  }

  permissions: {
    clipboard: boolean
    shell: boolean
    filesystem: boolean
  }

  createdAt: number
}
```

---

## 6.2 自动化系统

### 核心模型

```text
Trigger
↓
Condition
↓
Action
```

### Trigger（触发器）

支持：

- 全局快捷键
- 页面加载
- URL变化
- 菜单点击
- 定时任务
- 系统托盘

### Condition（条件）

支持：

- URL匹配
- 正则匹配
- 页面元素存在
- 当前应用判断
- 系统平台判断

### Action（动作）

支持：

#### 页面动作

- 点击元素
- 聚焦元素
- 输入文本
- 执行JS
- 等待元素

#### 系统动作

- 读取剪贴板
- 写入剪贴板
- 打开应用
- 打开文件
- 执行Shell
- 系统通知

---

## 6.3 用户脚本系统

### 目标

类似：

- Tampermonkey
- Userscript

但增强：

```text
网页能力
+
桌面能力
```

### 脚本语言

```text
JavaScript / TypeScript
```

不暴露 Rust。

### 示例

```ts
workflow.onShortcut("Ctrl+Alt+A", async () => {

  const text = await clipboard.readText()

  await page.focus("textarea")

  await page.type(`
解释以下内容：

${text}
`)

})
```

---

## 6.4 图形化自动化编辑器

### 普通用户模式

采用：

```text
步骤流
```

而非复杂节点图。

### 示例

```text
[触发]
Ctrl+Alt+A

[条件]
当前网址包含 chatgpt.com

[步骤]
1. 读取剪贴板
2. 聚焦输入框
3. 输入文本
4. 点击发送
```

### 高级模式

切换：

```text
脚本编辑器
```

---

## 6.5 页面增强系统

### JS Bridge

前端注入：

```js
window.__BANDOO__
```

用于：

- DOM操作
- 路由监听
- 和 Rust 通信

### 路由监听

支持：

- SPA
- React Router
- Vue Router
- History API

---

## 6.6 菜单增强系统

### 功能

根据：

- URL
- 路由
- 正则

动态显示菜单。

### 示例

```text
AI
├── 翻译
├── 解释
├── 总结
└── 代码优化
```

---

# 7. 权限系统

## 权限等级

| 权限 | 描述 |
|---|---|
| 页面权限 | DOM操作 |
| 剪贴板权限 | 读取剪贴板 |
| 文件权限 | 文件系统 |
| Shell权限 | 执行命令 |
| 网络权限 | 请求接口 |

### 原则

```text
默认最小权限
```

---

# 8. MVP 范围

## 第一阶段

### 基础运行时

- WebApp 创建
- 独立窗口
- 托盘
- 快捷键
- JS 注入

---

## 第二阶段

### 自动化能力

- Action系统
- 工作流
- 剪贴板
- 菜单增强

---

## 第三阶段

### 用户脚本

- Script API
- TS支持
- 脚本管理

---

## 第四阶段

### 图形化编辑器

- 拖拽
- 录制
- 元素选择器

---

# 9. 后续规划

## AI能力

支持：

- Prompt模板
- AI快捷操作
- OCR
- 截图分析
- 本地模型（Ollama）

## 插件生态

未来：

```text
Plugin SDK
```

## 云同步

支持：

- 工作流同步
- 配置同步
- 脚本同步

---

# 10. 产品定位总结

```text
比 PWA 更强
比 Electron 更轻
比 浏览器插件 更自由
```

---

# 11. 产品一句话

```text
将任意网页变成可自动化、可扩展的桌面应用。
```
