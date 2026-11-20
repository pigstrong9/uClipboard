# uClipboard 设计文档

- 日期：2026-11-20
- 状态：设计已确认；M0 静态原型已完成（2026-11-20）
- 许可证：Apache-2.0
- 设计参考：同类剪贴板管理器的通用交互模式；不使用任何第三方产品的代码、Logo、图标等资产

## 1. 背景与定位

uClipboard 是一个开源、跨平台、只做剪贴板管理的桌面应用：自动记录文本、图片、文件三类剪贴板历史，支持搜索、收藏、置顶、快捷粘贴，界面采用通用的剪贴板管理器布局。

调研发现同类开源项目：

| 项目 | 许可证 | 结论 |
|---|---|---|
| EcoPaste | Apache-2.0 | 可借鉴实现，必要时可复用代码并保留声明 |
| TieZ | GPL-3.0 | 只读思路；复制其代码将要求本项目整体 GPL-3.0 |
| 某商业剪贴板工具（闭源） | 闭源 | 仅作市场调研，不提取、不反编译、不使用素材 |

本项目采用 Apache-2.0，便于与 EcoPaste 生态互相借鉴。

## 2. 范围

### 2.1 v1 交付

- 历史记录：文本、图片、文件三类；全部/文本/图像/文件/收藏五个分类；实时搜索；收藏；置顶；删除；清空
- 快捷粘贴：全局快捷键唤起；Enter/双击/数字键粘贴到刚才的应用；Ctrl/Cmd+C 仅复制
- 选择与临时中转：单击条目切换高亮/多选（无需修饰键），再点右栏按钮把已选条目加入右下角「临时中转」浮层，集中搜索、复制全部、粘贴全部、收藏、移除
- 系统集成：托盘常驻、开机自启、单实例、历史上限与保留天数、忽略应用、密码管理器隐私标记识别、纯文本粘贴选项
- 体验：亮/暗/跟随系统主题、中英文、GitHub Releases 自动更新、完整键盘操作
- 视觉：清晰高效的剪贴板管理器布局，自有图标与品牌

### 2.2 v1 非目标

- 不做高级合并编辑（自定义分隔符、混排编辑）；临时中转按加入顺序复制/粘贴
- 不做完整的紧凑主面板切换；临时中转在 M0 为应用内浮层，Tauri 版为独立置顶窗口
- 不做图片/文件拖拽到其他应用
- 不做跨设备同步、云账号、分享
- 不单设「链接」分类，不做 URL/邮箱徽标识别
- 不做 OCR、翻译、模板、标签、分组
- 不做 Windows/macOS 代码签名与公证（文档说明系统提示）

### 2.3 视觉设计原则

- 布局、间距、圆角、配色层级、字号体系、明暗主题按通用剪贴板管理器模式逐项检查
- 图标使用 Lucide（ISC 许可证）或自绘 SVG，保持同风格，不使用第三方美术素材
- 应用名称与 Logo 为 uClipboard 自有品牌
- 视觉对照仅使用公开产品截图做内部参考，不将第三方截图或素材提交到仓库

## 3. 技术栈

| 层 | 选择 |
|---|---|
| 桌面框架 | Tauri 2 |
| 前端 | React 19 + TypeScript + Vite |
| 样式 | Tailwind CSS + Radix 原语 |
| 状态 | Zustand |
| 后端 | Rust |
| 剪贴板读取 | clipboard-rs（轮询） |
| 存储 | rusqlite（bundled SQLite）+ appData 图片文件 |
| 粘贴模拟 | enigo |
| 官方插件 | global-shortcut、autostart、single-instance、updater、log |
| 托盘 | Tauri 内置 tray API |
| 测试 | cargo test、Vitest + Testing Library |
| 构建发布 | GitHub Actions + tauri-action |

## 4. 架构与目录

```
uClipboard/
├─ src/                        # React 前端
│  ├─ pages/Clipboard.tsx      # 主界面：头部 + 分类 + 搜索 + 列表
│  ├─ pages/Settings.tsx       # 设置页
│  ├─ components/              # 列表项、文本/图片/文件卡片、预览、菜单、toast
│  ├─ stores/                  # zustand：列表、设置、UI 状态
│  ├─ i18n/                    # zh-CN / en-US
│  └─ styles/                  # 主题 token、全局样式
├─ src-tauri/
│  ├─ src/
│  │  ├─ clipboard/            # 轮询采集、内容模型、去重、抑制标记
│  │  ├─ storage/              # SQLite、迁移、检索、清理
│  │  ├─ platform/             # 粘贴模拟、前台应用、隐私格式
│  │  ├─ commands.rs           # IPC 命令
│  │  ├─ settings.rs           # 设置持久化
│  │  ├─ tray.rs               # 托盘菜单
│  │  └─ lib.rs / main.rs
│  ├─ icons/                   # 自有应用图标
│  ├─ tauri.conf.json
│  └─ Cargo.toml
├─ prototypes/                 # M0 静态 HTML 原型
├─ docs/superpowers/specs/     # 设计文档
├─ .github/workflows/          # 测试与三平台发布
├─ README.md
└─ LICENSE                     # Apache-2.0
```

运行形态：主窗口（无边框、自绘标题栏、可拖动）+ 独立的「临时中转」置顶小窗 + 系统托盘。开机静默驻留，快捷键唤起主窗口，再次触发或失焦/粘贴后隐藏。单实例运行。

## 5. 数据模型

### 5.1 SQLite

```sql
CREATE TABLE items (
  id          TEXT PRIMARY KEY,   -- UUID v7，时间有序，为将来同步预留
  kind        TEXT NOT NULL,      -- 'text' | 'image' | 'files'
  content     TEXT NOT NULL,      -- 文本正文 / 图片相对路径 / 文件路径 JSON 数组
  extra       TEXT,               -- JSON：html、图片宽高、文件数量等
  preview     TEXT,               -- 列表预览截断文本
  hash        TEXT NOT NULL,      -- 主内容 SHA-256
  size        INTEGER NOT NULL,   -- 字符数 / 字节数 / 文件数
  source_app  TEXT,               -- 复制时前台进程，仅用于忽略名单
  favorite    INTEGER NOT NULL DEFAULT 0,
  pinned      INTEGER NOT NULL DEFAULT 0,
  copy_count  INTEGER NOT NULL DEFAULT 1,
  created_at  INTEGER NOT NULL,   -- unix ms
  updated_at  INTEGER NOT NULL,   -- 最后使用/最后复制时间
  deleted_at  INTEGER             -- 软删除，预留同步
);
CREATE INDEX idx_items_order ON items(pinned DESC, updated_at DESC);
CREATE INDEX idx_items_kind  ON items(kind, favorite, updated_at DESC);
```

- 图片：原图存 `appData/images/{hash}.png`，缩略图存 `appData/images/{hash}_thumb.webp`（宽 300px）
- 检索：`LIKE` 模糊匹配文本正文与文件名，`%`、`_` 转义；结果截断 200 条；前端 120ms 防抖
- 排序：置顶优先，组内 `updated_at` 倒序

### 5.2 设置（`appData/config.json`）

| 键 | 默认值 | 说明 |
|---|---|---|
| hotkey | `Alt+V`（macOS `Option+V`） | 全局快捷键 |
| autoStart | 打包版 true，开发环境 false | 开机自启 |
| hideOnBlur | true | 失焦自动隐藏 |
| theme | system | system / light / dark |
| language | system | system / zh-CN / en-US |
| historyLimit | 500 | 非收藏、非置顶项上限 |
| retentionDays | 30 | 保留天数，0 表示永久 |
| recordText / recordImages / recordFiles | true | 分类采集开关 |
| minTextLength | 3 | 短于此长度的文本不记录 |
| ignoredApps | [] | 进程名列表 |
| pastePlainText | false | 粘贴时去格式 |
| autoUpdate | true | 自动检查更新 |
| monitorPaused | false | 暂停采集 |
| maxTextBytes | 5 MB | 超限文本丢弃 |
| maxImageBytes | 20 MB | 超限图片丢弃 |

## 6. 核心流程

### 6.1 采集

每 300ms 读取剪贴板（文本、HTML、图片、文件列表），按优先级 files > image > text 归为一条：

1. 计算主内容 hash（文本字节 / 图片原始字节 / 排序后的文件路径 JSON）；与上次相同则忽略
2. 命中自身写入抑制标记（1.5s 内）则忽略
3. 命中忽略应用、暂停状态、分类开关关闭、超出大小上限则丢弃
4. 命中系统隐私标记则丢弃（见第 9 节）
5. hash 已存在：更新 `updated_at=now`、`copy_count+1`，不新增行
6. 否则入库；图片落盘并生成缩略图；推送事件给前端
7. 达到历史上限后按清理规则删除

### 6.2 使用即置顶

- 粘贴（Enter/双击/数字键）或复制（Ctrl/Cmd+C）任一动作：`updated_at=now`、`copy_count+1`，该条立即移动到第一位
- 置顶组优先，组内仍按 `updated_at` 排序
- 外部再次复制已有内容：命中 hash，直接把旧记录移到第一位，不产生重复行
- 自身写回剪贴板由抑制标记阻断，不产生新记录；置顶只由动作本身触发一次
- 重新打开窗口：清空搜索、选中第一条，便于连续取用
- 收藏页同样按 `updated_at` 排序；置顶只在「全部/文本/图像/文件」分类生效

### 6.3 粘贴

1. 前端调用 `paste_item(id)`
2. Rust 将条目原始格式写回剪贴板（文本、HTML、图片、文件列表）；开启纯文本时只写文本
3. 设置自身写入抑制标记
4. 隐藏窗口，等待 120ms 焦点返回
5. enigo 发送 `Ctrl+V` / `Cmd+V`
6. 失败则降级为仅复制，并 toast「已复制，请手动粘贴」

### 6.4 清理

- 启动时与每 6 小时执行一次
- 超出 `historyLimit` 或早于 `retentionDays` 的非收藏、非置顶项删除
- 同时删除对应图片与缩略图文件
- 用户删除为立即删除（含文件）；清空历史需二次确认

## 7. 界面规格

### 7.1 主窗口

默认 880×660，最小 720×520，可缩放。无边框、自绘标题栏、圆角（平台支持时）。

```
┌──────────────────────────────────────────────┐
│ uClipboard        [🔍 搜索…]        ⚙ ⋯  ✕  │
├──────────────────────────────────────────────┤
│ 全部   文本   图像   文件   ★收藏(6)          │
├──────────────────────────────────────────────┤
│ 文本预览最多 4 行，超长可展开…        3 分钟前 │
│ 440 字符                                  1  │
│ 图片缩略图  1205×903                       2 │
│ 文件名列表 ×4           4 个文件           3 │
├──────────────────────────────────────────────┤
│ 监控中 · 共 128 条      [设置] [清空历史]      │
└──────────────────────────────────────────────┘
```

- 头部：左上角仅「剪贴板」文字、搜索框（唤起时自动聚焦）、暂停状态标识、设置/更多/关闭按钮
- 分类行：全部/文本/图像/文件/收藏（仅收藏显示数量），左右方向键切换
- 列表：虚拟滚动；前 9 条右侧显示序号；选中项蓝色描边；悬停显示粘贴/收藏/删除图标
- 条目底部左侧为相对时间，右侧为字符数/图片尺寸/文件数量
- 长文本可展开/收起；点击图片打开大图弹窗；`Space` 打开全文/大图预览
- 右键菜单：粘贴、复制、收藏/取消收藏、置顶/取消置顶、删除
- 右侧独立按钮栏：复制、粘贴、多选/临时中转三个圆形按钮；滚动条位于列表右缘、按钮栏左侧
- 按钮栏底部：设置、清空历史
- 左下角状态标签：监控状态与总条数
- 选择方式：单击条目切换高亮，多选无需修饰键；点击右栏第三个按钮把已选条目加入「临时中转」
- 独立「临时中转」小窗（M0 用页面级浮窗模拟）：搜索、复制全部、粘贴全部、收藏全部、移除、清空、关闭
- 空状态、加载骨架、粘贴失败 toast

### 7.2 视觉 token

| Token | 亮色 | 暗色 |
|---|---|---|
| 主色 accent | `#3B82F6` | `#60A5FA` |
| 窗口背景 | `#F5F6F8` | `#1B1C1E` |
| 卡片/列表背景 | `#FFFFFF` | `#232427` |
| 主文字 | `#1F2329` | `#E6E7E9` |
| 次要文字 | `#6B7280` | `#9AA0A6` |
| 边框 | `#E5E7EB` | `#33353A` |
| 悬停背景 | `#F0F4FF` | `#2A2C30` |
| 选中描边 | `#3B82F6` | `#60A5FA` |

- 字体：`-apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif`；文本预览用等宽字体栈
- 字号：分类 14px、条目正文 14px、元信息 12px、标题 13px
- 圆角：窗口 12px、卡片 8px、按钮 8px、圆形按钮 999px
- 图标：Lucide + 自绘 SVG

### 7.3 键盘与鼠标

| 键 | 行为 |
|---|---|
| `↑↓` / `Tab` | 上下选择 |
| `←→` | 切换分类 |
| `Enter` | 粘贴选中项 |
| 双击条目 | 粘贴到刚才的应用 |
| `Ctrl/Cmd+C` | 仅复制 |
| `1-9`（列表聚焦时） | 直接粘贴对应序号 |
| `Space` | 预览（大图/全文） |
| `Ctrl+D` | 收藏/取消收藏 |
| `Ctrl+P` | 置顶/取消置顶 |
| `Delete` / `Cmd+Backspace` | 删除选中项 |
| `/` 或字母（列表聚焦时） | 聚焦搜索并输入 |
| `Esc` | 关闭浮层/预览 → 清空搜索 → 隐藏窗口 |

搜索框聚焦时所有按键正常输入。单击切换高亮/多选，双击粘贴；点击右栏按钮将高亮条目加入临时中转，浮窗内可复制全部、粘贴全部、收藏全部。

### 7.4 设置页

同窗口内切换，左侧分区：

- 基本：快捷键、开机自启、失焦隐藏、纯文本粘贴、主题、语言
- 历史：上限、保留天数、三类采集开关、最短文本长度、暂停采集
- 隐私：忽略应用管理（内置 KeePass/1Password/Bitwarden 等建议项）、隐私标记说明、清空全部数据
- 关于：版本、许可证、检查更新、打开日志目录

## 8. 系统集成

- 全局快捷键默认 `Alt+V`（macOS `Option+V`），设置页可改；注册失败时提示冲突并保留托盘入口
- 托盘菜单：显示主窗口、暂停/恢复监控、清空历史、设置、退出
- 单实例：再次启动唤起已有窗口
- 临时中转窗口：独立的 Tauri WebviewWindow，无边框、置顶、不抢焦点，默认定位在主窗口右下角外侧；与主窗口共享 Rust 状态并通过事件同步；关闭中转窗不退出应用，也不影响主窗口
- 开机自启默认开启（打包版）；开发环境默认关闭
- 唤起：窗口置于鼠标所在显示器中央、置顶显示、聚焦搜索框、清空搜索、选中第一条
- 隐藏：`Esc`、粘贴完成、失焦（可关）
- 自动更新：启动时及每 24 小时检查 GitHub Releases；设置页手动检查；下载安装后重启

### 平台差异

| 平台 | 采集 | 粘贴 | 前台应用 | 备注 |
|---|---|---|---|---|
| Windows | clipboard-rs 轮询 | enigo 发送 Ctrl+V | GetForegroundWindow + 进程名 | 读取剪贴板隐私格式 |
| macOS | clipboard-rs 轮询 | enigo 发送 Cmd+V，需辅助功能授权 | NSWorkspace 前台应用 | 设置页检测授权并提供跳转 |
| Linux X11 | clipboard-rs 轮询 | enigo 模拟粘贴 | 尽力获取 | — |
| Linux Wayland | 依赖系统能力 | 失败则降级仅复制 | 通常不可用，记录为空 | 文档说明限制 |

## 9. 隐私与安全

- 仅自动更新访问 GitHub，其余无网络请求；无账号、无云端
- Windows：读取 `ExcludeClipboardContentFromMonitorProcessing`、`CanIncludeInClipboardHistory=0` 等隐私格式，命中则不记录
- macOS：识别 `org.nspasteboard.ConcealedType`、`org.nspasteboard.TransientType`、`org.nspasteboard.AutoGeneratedType`
- 忽略应用按进程名匹配，默认建议密码管理器名单
- 数据静态存储不做加密，README 明确说明；敏感内容依赖忽略名单与隐私标记
- 清空历史与删除条目会同步删除图片文件

## 10. 错误处理与降级

- 剪贴板读取失败：记录日志，下个周期重试，界面底部显示监控异常提示
- 粘贴模拟失败（权限/Wayland）：降级仅复制 + toast 提示
- 数据库写入失败：toast 提示并记录日志，应用保持运行
- 图片损坏或文件已不存在：显示占位状态，仍可复制原始路径
- 更新检查失败：设置页显示失败原因，不打扰主流程
- 日志：tauri-plugin-log 写入 `appData/logs`，按大小轮转

## 11. 测试策略

- Rust 单测：存储 CRUD、去重与使用即置顶、超限/超期清理、忽略应用匹配、设置读写与迁移
- Rust 集成：剪贴板源抽象为 trait，用 mock 验证轮询、抑制标记、隐私格式过滤
- 前端：Vitest + Testing Library 覆盖键盘交互、搜索过滤、主题切换、i18n 键完整性
- 手动冒烟（三平台）：文本/图片/文件采集、重复复制置顶、粘贴到记事本/浏览器、快捷键唤起、托盘、自启、Wayland 降级、macOS 授权、更新检查
- UI 验证：M0 原型对照截图确认；实现后在真机实际粘贴验证
- 自动化端到端（tauri-driver）v1 不做，避免跨平台稳定性问题

## 12. CI/CD 与发布

- GitHub Actions：
  - `ubuntu-latest`：前端测试 + cargo test
  - `windows-latest`、`macos-latest`、`ubuntu-22.04`：构建
  - 打 `v*` tag：tauri-action 创建草稿 Release
- 产物：Windows NSIS/MSI、macOS dmg、Linux AppImage/deb
- 自动更新：Tauri 签名密钥存入 GitHub Secrets；v1 不做 Windows/macOS 代码签名，README 说明 SmartScreen/Gatekeeper 提示及绕过方式
- 许可证：Apache-2.0

## 13. 里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| M0 | 静态 HTML 原型：双主题、真实 mock 数据、完整布局 | 与参考截图并排对比确认视觉与交互 |
| M1 | Tauri 骨架 + SQLite + 轮询采集 + 列表/搜索/收藏/置顶/删除 | Windows 本机可用，数据持久化 |
| M2 | 全局快捷键 + 自动粘贴 + 托盘 + 自启 + 忽略应用 + 隐私标记 + 多选/临时中转窗口 | 真机端到端粘贴通过 |
| M3 | 主题/i18n/自动更新/三平台打包与 CI | CI 三平台构建成功 |
| M4 | README、LICENSE、v0.1.0 首发 | 发布页可下载 |

## 14. 风险与对策

| 风险 | 对策 |
|---|---|
| macOS 辅助功能授权导致粘贴不可用 | 状态检测 + 引导跳转；失败降级仅复制 |
| Wayland 全局快捷键与粘贴受限 | 托盘兜底；粘贴降级仅复制；README 说明 |
| Linux 文件剪贴板格式差异 | 真机验证；解析失败时保存文本路径 |
| 全局快捷键冲突 | 注册失败提示并允许改绑 |
| 大图片/长文本造成卡顿 | 大小上限、缩略图、列表虚拟滚动 |
| 自动更新密钥管理 | 仅存 GitHub Secrets，README 说明签名流程 |
| 视觉还原偏差 | M0 原型先对照确认，再进入 Tauri 实现 |

## 15. 前置环境与开工条件

- Windows 开发机需安装：rustup（stable MSVC 工具链）、Visual Studio Build Tools（C++ 桌面开发）、WebView2 Runtime
- Node.js 与 npm 已具备；pnpm 可选安装
- 系统级安装执行前需用户确认

## 16. 法务边界

- 不提取、不反编译任何第三方闭源产品
- 不使用第三方产品的 Logo、图标、截图等美术素材；图标全部使用开源图标库或自绘
- 不复制无许可证项目的代码
- 项目以 Apache-2.0 发布，第三方依赖保留各自许可证声明
