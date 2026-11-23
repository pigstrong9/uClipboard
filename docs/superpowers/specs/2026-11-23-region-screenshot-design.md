# 区域截图功能设计

日期：2026-11-23
状态：已确认方案，进入实现

## 目标

提供一个接近微信截图的最小区域截图：全局快捷键唤起全屏覆盖层，拖动选择区域，松开鼠标后自动裁剪并写入系统剪贴板。不做标注、保存文件、贴图或编辑工具栏。

## 触发

- 只做全局快捷键，不在主界面增加按钮。
- 新增设置项 `screenshotHotkey`，默认 `Alt+A`，可在设置页“基本”中修改。
- 启动时注册 `hotkey`（主窗）和 `screenshotHotkey`；若截图快捷键被占用，自动退到 `Ctrl+Shift+A`、写回配置并提示用户；兜底也失败时只记录日志并提示手动修改。
- 修改设置时分别注销/重注册；注册失败回滚并返回错误码。两个快捷键相同则拒绝保存。

## 用户流程

1. 按截图快捷键。
2. 隐藏主窗与中转窗，等待约 200ms。
3. 抓取整个虚拟桌面并拼成一张物理像素图，保存为 `images/shot_<uuid>.png`。
4. 显示无边框、置顶、跨全部显示器的覆盖层窗口。
5. 用户拖动选区；选区外压暗，显示选区物理像素尺寸。
6. 松开鼠标（选区宽高至少 2px）后裁剪原图、编码 PNG、写入剪贴板，关闭覆盖层并删除临时图。
7. `Esc` 或右键取消，删除临时图并关闭覆盖层。
8. 剪贴板监控会照常把截图收进历史；监控暂停时只留在剪贴板。

## 架构

### 后端

新增 `src-tauri/src/screenshot.rs`：

- `start(app)`：隐藏窗口、抓屏、落临时文件、创建覆盖层窗口、写入会话状态。
- `capture_region(x, y, width, height)`：读取临时图、裁剪、写剪贴板、关闭窗口、清理。
- `cancel()`：关闭窗口并清理。
- `session()`：给覆盖层返回临时图路径与像素尺寸。
- `ready()`：覆盖层图片加载完成后显示并聚焦窗口。

抓屏使用 `xcap`，覆盖 Windows、macOS、Linux X11/Wayland。多个显示器按各自物理原点拼到一个画布；macOS 将 monitor 逻辑原点乘以缩放因子后再拼。总像素超过 1.2 亿返回 `screenshot.too_large`。

`AppState` 增加 `screenshot: Mutex<Option<ScreenshotSession>>`。会话包含临时图路径、画布尺寸与物理原点。创建窗口失败、窗口销毁、取消、完成时都要清空会话并删除临时文件。

### 覆盖层窗口

- 新增 `screenshot.html` + `src/screenshot.tsx` + `src/ScreenshotApp.tsx`。
- 窗口 label 为 `screenshot`，无边框、置顶、跳过任务栏、不可缩放。
- 使用 `PhysicalPosition` / `PhysicalSize` 覆盖虚拟桌面物理范围。
- 窗口创建后立即显示，图片加载前显示准备提示；`screenshot_ready` 负责加载完成后再次聚焦。
- 覆盖层图片通过 asset protocol 加载。
- 前端用 `naturalWidth / getBoundingClientRect().width` 把 CSS 选区换算为原图像素，再调用 `capture_region`。
- 临时截图可能被 WebView 短暂占用，关闭覆盖层后带重试删除，避免残留。

### 快捷键

`on_shortcut` 用 `Shortcut::id()` 与设置中的两个快捷键比较：截图快捷键优先，否则按主窗快捷键切换显示。

### 错误码

- `screenshot.busy`
- `screenshot.capture_failed`
- `screenshot.too_large`
- `screenshot.window_failed`
- `screenshot.session_missing`
- `screenshot.write_failed`
- `screenshot.hotkey_fallback`
- `screenshot.hotkey_failed`
- `settings.hotkey_conflict`

快捷键触发失败时显示主窗并 emit `screenshot-error`；前端翻译后 toast。覆盖层内部失败时在覆盖层显示错误并允许 `Esc` 关闭。

## 已知限制

- 本机只能实测 Windows；macOS/Linux 保证编译与逻辑一致。
- macOS 需要“屏幕录制”权限；Wayland 的窗口定位与置顶由桌面环境决定，可能无法跨屏覆盖。
- macOS 混合缩放比例的多显示器，物理原点按缩放因子近似，可能存在偏移；单屏或统一缩放下正常。

## 测试

- Rust：虚拟桌面边界计算、选区裁剪边界、设置默认值与快捷键冲突。
- 前端：CSS 选区到图片像素换算、覆盖层 Esc/右键取消、加载完成前不显示窗口。
- 真机：Windows 下全局快捷键、拖选、剪贴板图片尺寸、Esc 取消、覆盖层不截入自身。
