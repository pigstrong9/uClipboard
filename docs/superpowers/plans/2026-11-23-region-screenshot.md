# 区域截图实施计划

1. **依赖与模型**
   - 添加 `xcap` 依赖。
   - `Settings` 增加 `screenshot_hotkey`（默认 `Alt+A`）及归一化。
   - `AppState` 增加截图会话。
   - 先写设置默认值/归一化测试。

2. **后端截图模块**
   - 新增 `screenshot.rs`：虚拟桌面布局、抓屏、临时文件、窗口创建、会话命令。
   - 先写布局与裁剪边界的纯函数测试。
   - 注册命令；覆盖层销毁时清理。

3. **快捷键**
   - 启动时注册两个快捷键。
   - `on_shortcut` 按 id 分流。
   - 设置更新时成对注册/回滚；重复快捷键返回 `settings.hotkey_conflict`。
   - 失败时显示主窗并 emit `screenshot-error`。

4. **前端覆盖层**
   - 新增 `screenshot.html`、`src/screenshot.tsx`、`src/ScreenshotApp.tsx`。
   - Vite 增加多页入口。
   - 实现选区、压暗、尺寸标签、Esc/右键取消、加载完成 ready。
   - 先写 `selectionToPixels` 纯函数测试。

5. **设置与错误**
   - 前端 `Settings` 类型、设置页第二个快捷键输入、i18n。
   - 错误码映射、`screenshot-error` 监听。
   - capabilities 增加 `screenshot` 窗口。

6. **工程与验证**
   - Linux CI 补 xcap 所需系统依赖。
   - `npm run build`、`npm test`、`cargo fmt/clippy/test`。
   - Windows 真机验证：Alt+A → 拖选 → 剪贴板图片尺寸正确；Esc 取消；覆盖层不截入自身。
