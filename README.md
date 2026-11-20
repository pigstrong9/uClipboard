# uClipboard

跨平台开源剪贴板管理器，基于 Tauri 2 + React + TypeScript。

当前进度：M0 静态 HTML 原型（`prototypes/`），尚未接入真实系统剪贴板。

## 预览原型

```bash
node prototypes/serve.js
```

打开 http://localhost:4173 。可选参数：`?theme=light`、`?theme=dark`、`?tab=favorite`、`?settings=1`。

刷新页面会恢复匿名 mock 数据，原型不做持久化。

## 功能规划

- 文本 / 图片 / 文件三类剪贴板历史
- 搜索、收藏、置顶、删除
- 全局快捷键与自动粘贴
- 托盘、开机自启、历史上限、忽略应用
- 亮/暗主题、中英文、自动更新

## 文档

- 设计文档：`docs/superpowers/specs/2026-11-20-uclipboard-design.md`
- M0 验收：`docs/superpowers/plans/2026-11-20-uclipboard-m0-acceptance.md`

## 许可证

Apache-2.0。第三方图标声明见 `NOTICE`。

uClipboard 是独立开源项目，与任何第三方启动器或剪贴板产品无关联。
