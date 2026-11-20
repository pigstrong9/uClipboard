# uClipboard M2/M3 验收记录

- 日期：2026-11-20
- 分支：`master`
- 环境：Windows 11（10.0.26300），WebView2，Rust stable-msvc，Node 24
- 结论：M2/M3 功能实现完成，Windows 真机验证通过

## 自动化

| 项目 | 结果 |
|---|---|
| `npm run build`（tsc + Vite，双页面） | 通过 |
| `cargo test` | 17/17 通过 |
| `cargo fmt --check` | 通过 |
| `tauri build --debug` MSI | 通过 |
| `tauri build --debug --bundles nsis` | 通过，5.05 MiB |
| updater 配置覆盖 + 签名 | 通过，生成 `.exe.sig` |
| 第二实例启动 | 只保留 1 个进程，已有主窗口被唤起 |
| `--autostart` 启动 | 主窗口保持隐藏，托盘与快捷键驻留 |
| `Alt+V` 注册 | 外部再次注册返回 1409，确认已被应用占用 |
| 无效快捷键 | `NotAKey` 返回注册错误，旧快捷键保持 `Alt+V` |
| 窗口居中 | 鼠标所在显示器工作区中心，误差 0.5px |
| 外部复制文本 | 采集成功；A→B→A 后 A 回到顶部且 copy_count 递增 |
| 暂停/恢复 | 暂停期间复制不入库；恢复后立即复制可正常入库 |
| 来源应用 | 从 Notepad 复制记录 `Notepad.exe` |
| 忽略应用 | 加入 `Notepad.exe` 后复制不入库；移除后恢复采集 |
| 隐私标记 | 原生格式 `ExcludeClipboardContentFromMonitorProcessing` 与 `CanIncludeInClipboardHistory=0` 均不入库 |
| 富文本 | CF_HTML 采集为 HTML extra；复制回写 HTML + 纯文本 |
| 纯文本粘贴 | 开启后只写纯文本，不写 HTML |
| 单图复制 | 写回 1352×737 图片成功 |
| 多图合并 | 两张 1352×737 + 440×282 合并为 1352×1027 |
| 自动粘贴 | 主界面双击条目，目标 WinForms 文本框收到完整文本 |
| 中转粘贴 | 中转窗双击条目，目标收到文本；中转窗保持可见 |

产物：

- `src-tauri/target/debug/bundle/msi/uClipboard_0.1.0_x64_en-US.msi`
- `src-tauri/target/debug/bundle/nsis/uClipboard_0.1.0_x64-setup.exe`
- `src-tauri/target/debug/bundle/nsis/uClipboard_0.1.0_x64-setup.exe.sig`

## 界面与状态验证（WebView2 CDP）

| 场景 | 结果 |
|---|---|
| 设置页打开 / 关闭 / Esc | 通过 |
| 基本、历史、隐私、关于四个分区 | 通过 |
| 语言切换即时生效并持久化 | 通过，重启后保持 |
| 主题 system/light/dark 即时生效并持久化 | 通过，主窗口与中转窗同步 |
| 暂停监控状态 | 状态栏显示“已暂停”，恢复后显示“监控中” |
| 忽略应用名单 | 保存、去重、空行过滤、持久化通过；测试后恢复为空 |
| 快捷键改绑 | `Ctrl+Alt+V` 保存成功；无效值拒绝并保留旧值；已恢复 `Alt+V` |
| 开机自启开关 | 配置持久化通过；开发模式按设计不写入系统启动项；`--autostart` 静默启动已验证 |
| 托盘菜单清空 | 改为唤起主窗口并走二次确认，避免误清空 |
| 中转窗口 | 打开、关闭、主题/语言同步、主窗口保持可见 |
| 右键菜单 | 粘贴、复制、收藏、置顶、删除五项显示正常 |
| 更新检查 | 无可用 Release 时显示错误 toast，不影响主流程 |
| 单实例恢复 | 第二个实例退出并唤起隐藏的主窗口 |

## 清理与数据

- 删除条目、清空历史和定时清理会删除孤立图片与缩略图（单测覆盖）。
- 清理测试产生的中转选择，未清空用户历史。
- 测试后配置恢复为：主题 system、语言 system、快捷键 Alt+V、未暂停、忽略名单为空、开发模式自启关闭。

## 未覆盖

- 解锁后已补测外部复制采集、暂停/恢复、来源应用、忽略应用、隐私标记、富文本、单图复制、多图合并、主窗口自动粘贴和中转窗粘贴。
- 托盘菜单通过代码路径与底层命令验证，未做鼠标点击录屏。
- 自动更新下载安装需要真实 Release；已用本地私钥验证签名产物，未接入真实 Release 仓库。
- macOS 辅助功能授权、Wayland 降级尚未真机验证。
- 开机自启仅验证配置与插件接入，未在打包安装后重启系统验证。
