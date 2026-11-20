# uClipboard

跨平台开源剪贴板管理器，基于 Tauri 2 + React + TypeScript。

当前进度：**M1 核心版 + M2 粘贴/快捷键/托盘/临时中转**——真实剪贴板采集、SQLite 持久化、搜索/收藏/置顶/删除/复制，双击自动粘贴、`Alt+V` 唤回、托盘常驻、多选与独立临时中转窗口。

## 已实现（M1）

- 文本 / 图片 / 文件三类真实剪贴板采集
- 图片落盘、300px 缩略图与原始图预览
- SQLite 持久化，重启后数据恢复
- 搜索、收藏、置顶、删除、复制回剪贴板
- 双击条目自动粘贴到刚才的应用，并隐藏剪贴板窗口（Enter 同样粘贴）
- `Alt+V` 切换显示/隐藏主窗口；最小化时按 `Alt+V` 也会恢复并聚焦
- 主窗口不显示任务栏图标；托盘图标保留作兜底，可显示窗口或退出
- 单击条目多选，右栏按钮打开独立的置顶「临时中转」窗口；支持合并复制文本/文件、收藏全部、搜索、关闭
- 中转窗口内双击条目直接粘贴到目标应用，中转窗口保持打开
- 同一内容自动去重，并移动到非置顶列表顶部
- 亮 / 暗主题
- 9 项 Rust 单元测试覆盖存储、去重、清理、搜索与采集归一化

## 开发（Windows）

前置环境：Node.js、Rust（stable-msvc）、Visual Studio Build Tools（C++ 桌面开发工作负载）、WebView2 Runtime。

```bat
scripts\check.cmd   :: cargo check
scripts\test.cmd    :: cargo test
scripts\dev.cmd     :: npm run tauri dev
```

说明：Git Bash 自带的 `link.exe` 会抢占 MSVC 链接器，脚本会先加载 `vcvars64.bat` 再执行命令。GitHub Actions 使用原生 PowerShell，不需要这个包装。

数据位置：`%APPDATA%\com.uclipboard.app\uclipboard.db`，图片在其中的 `images/` 目录。

## M0 静态原型

`prototypes/` 保留 M0 的静态 HTML 原型，用于设计预览：

```bash
node prototypes/serve.js
```

打开 http://localhost:4173 。可选参数：`?theme=light`、`?theme=dark`、`?tab=favorite`、`?settings=1`。

## 后续

- M2 剩余：忽略应用、开机自启、来源应用
- M3：设置持久化、中英文、自动更新、三平台打包与 CI

## 文档

- 设计文档：`docs/superpowers/specs/2026-11-20-uclipboard-design.md`
- M0 验收：`docs/superpowers/plans/2026-11-20-uclipboard-m0-acceptance.md`
- M1 验收：`docs/superpowers/plans/2026-11-20-uclipboard-m1-acceptance.md`
- M2 粘贴增量验收：`docs/superpowers/plans/2026-11-20-uclipboard-m2-paste-acceptance.md`
- M2 临时中转验收：`docs/superpowers/plans/2026-11-20-uclipboard-m2-transfer-acceptance.md`

## 许可证

Apache-2.0。第三方图标声明见 `NOTICE`。

uClipboard 是独立开源项目，与任何第三方启动器或剪贴板产品无关联。
