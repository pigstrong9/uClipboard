# uClipboard M0 静态原型 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用纯静态 HTML/CSS/JS 复刻 uTools 剪贴板主界面，带真实 mock 数据、亮暗双主题与完整键盘交互，供视觉确认后进入 Tauri 实现。

**Architecture:** 单页应用，无构建步骤、无第三方依赖，`node prototypes/serve.js` 启动本地静态服务。界面由 `index.html` 骨架 + `styles.css` 主题 token + `mock-data.js` 数据 + `app.js` 渲染与交互组成。所有图标内联 SVG，字体走系统字体栈。

**Tech Stack:** HTML5、CSS 变量、原生 JavaScript（classic script，无 ES module）、Node.js 内置 `http` 模块。

**Spec:** `docs/superpowers/specs/2026-11-20-uclipboard-design.md`

## Global Constraints

- 不使用 uTools 的 Logo、图标、截图等美术素材；图标为 Lucide 风格内联 SVG 或自绘
- 不复制闭源或无许可证项目的代码
- 原型无网络请求：不引 CDN、不加载外部字体、不加载外部图片
- 界面文案为简体中文；i18n 接线留到 M3
- 主窗口尺寸 880×660
- 主色 `#368CFF`；亮暗 token 严格按 spec 第 7.2 节
- 字体：`-apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif`；文本预览用等宽字体栈
- 原型状态仅存内存，刷新即重置，不做 localStorage 持久化
- 提交身份使用一次性参数，不修改 git config：
  `git -c user.name="pig strong" -c user.email="29220747+pigstrong9@users.noreply.github.com" commit ...`
- 所有验证通过 browser-use MCP 的 `navigate_page` / `evaluate_script` / `take_screenshot` 完成
- 每个任务开始验证前刷新页面，确保内存状态从 mock 数据重新初始化

## File Structure

```
prototypes/
├─ serve.js        # 零依赖静态服务器，端口 4173
├─ index.html      # 页面骨架：标题栏、分类、列表、状态栏、弹窗容器
├─ styles.css      # 主题 token、布局、条目卡片、弹窗、toast
├─ mock-data.js    # 12 条 mock 记录（文本/图片/文件，含收藏与置顶）
└─ app.js          # 状态、排序过滤、渲染、键盘与鼠标交互
```

---

### Task 1: 静态服务器与窗口骨架

**Files:**
- Create: `prototypes/serve.js`
- Create: `prototypes/index.html`
- Create: `prototypes/styles.css`

**Interfaces:**
- Produces: 可访问的 `http://localhost:4173`；`data-theme` 属性驱动主题；`.app` 固定 880×660 窗口

- [ ] **Step 1: 写零依赖静态服务器**

```js
// prototypes/serve.js
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');

const root = __dirname;
const port = 4173;
const types = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
};

http
  .createServer((req, res) => {
    const urlPath = decodeURIComponent((req.url || '/').split('?')[0]);
    const filePath = path.join(root, urlPath === '/' ? 'index.html' : urlPath);
    if (!filePath.startsWith(root)) {
      res.writeHead(403).end('Forbidden');
      return;
    }
    fs.readFile(filePath, (err, data) => {
      if (err) {
        res.writeHead(404).end('Not found');
        return;
      }
      res.writeHead(200, {
        'Content-Type': types[path.extname(filePath)] || 'application/octet-stream',
      });
      res.end(data);
    });
  })
  .listen(port, () => console.log(`Prototype: http://localhost:${port}`));
```

- [ ] **Step 2: 写页面骨架**

```html
<!doctype html>
<html lang="zh-CN" data-theme="dark">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>uClipboard 原型</title>
    <link rel="stylesheet" href="styles.css" />
  </head>
  <body>
    <main class="app" id="app">
      <header class="titlebar" data-tauri-drag-region>
        <div class="brand">
          <span class="brand-logo" aria-hidden="true"><!-- 内联 SVG：剪贴板 --></span>
          <span class="brand-name">剪贴板</span>
          <button class="icon-btn" id="hideBtn" title="隐藏（原型）">×</button>
        </div>
        <label class="search" for="searchInput">
          <span class="search-icon" aria-hidden="true"><!-- 内联 SVG：放大镜 --></span>
          <input id="searchInput" type="text" placeholder="搜索…" autocomplete="off" />
        </label>
        <div class="title-actions">
          <button class="icon-btn" id="themeBtn" title="切换主题"><!-- 内联 SVG：太阳/月亮 --></button>
          <button class="icon-btn" id="settingsBtn" title="设置"><!-- 内联 SVG：齿轮 --></button>
          <button class="icon-btn" id="moreBtn" title="更多"><!-- 内联 SVG：三点 --></button>
        </div>
      </header>
      <nav class="tabs" id="tabs" aria-label="分类"></nav>
      <section class="list" id="list" tabindex="0" aria-label="剪贴板历史"></section>
      <footer class="statusbar">
        <span id="statusText">监控中 · 共 0 条</span>
        <div class="status-actions">
          <button class="text-btn" id="settingsTextBtn">设置</button>
          <button class="text-btn danger" id="clearBtn">清空历史</button>
        </div>
      </footer>
    </main>
    <div class="toast" id="toast" role="status" hidden></div>
    <div class="modal-backdrop" id="modal" hidden></div>
    <div class="context-menu" id="contextMenu" hidden></div>
    <div class="restore-hint" id="restoreHint" hidden>窗口已隐藏（原型）— 点击恢复</div>
  </body>
</html>
```

- [ ] **Step 3: 写主题 token 与窗口布局**

```css
/* prototypes/styles.css（节选，后续任务继续追加） */
:root {
  --accent: #368cff;
  --radius-window: 12px;
  --radius-card: 8px;
  --font-ui: -apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
  --font-mono: ui-monospace, "SF Mono", "Cascadia Mono", Consolas, monospace;
}

:root[data-theme="dark"] {
  --bg: #1b1c1e;
  --surface: #232427;
  --surface-2: #2a2c30;
  --text: #e6e7e9;
  --text-2: #9aa0a6;
  --border: #33353a;
  --hover: #2a2c30;
  --selected: #4c9bff;
}

:root[data-theme="light"] {
  --bg: #f5f6f8;
  --surface: #ffffff;
  --surface-2: #f0f4ff;
  --text: #1f2329;
  --text-2: #6b7280;
  --border: #e5e7eb;
  --hover: #f0f4ff;
  --selected: #368cff;
}

* { box-sizing: border-box; }

body {
  margin: 0;
  min-height: 100vh;
  display: grid;
  place-items: center;
  background: #0f1012;
  font-family: var(--font-ui);
  color: var(--text);
}

.app {
  position: relative;
  width: 880px;
  height: 660px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: var(--radius-window);
  background: var(--bg);
  border: 1px solid var(--border);
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.45);
}

.titlebar {
  height: 56px;
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 0 12px 0 8px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
}

.brand { display: flex; align-items: center; gap: 6px; flex: none; }
.brand-name { font-size: 13px; color: var(--text-2); }

.search {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 12px;
  border-radius: 8px;
  background: var(--bg);
  border: 1px solid var(--border);
}

.search input {
  flex: 1;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text);
  font: inherit;
  font-size: 14px;
}

.tabs {
  display: flex;
  gap: 4px;
  padding: 8px 16px 4px;
  background: var(--bg);
}

.list {
  flex: 1;
  overflow-y: auto;
  padding: 0 16px 12px;
  outline: 0;
}

.statusbar {
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-top: 1px solid var(--border);
  color: var(--text-2);
  font-size: 12px;
}

.icon-btn {
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-2);
  cursor: pointer;
}
.icon-btn:hover { background: var(--hover); color: var(--text); }
.brand-logo svg { width: 16px; height: 16px; color: var(--accent); }
.search-icon svg { width: 16px; height: 16px; color: var(--text-2); }
.icon-btn svg { width: 16px; height: 16px; }

.toast {
  position: fixed;
  left: 50%;
  bottom: 40px;
  transform: translateX(-50%);
  padding: 8px 16px;
  border-radius: 999px;
  background: rgba(30, 32, 36, 0.92);
  color: #fff;
  font-size: 13px;
  z-index: 90;
}
```

- [ ] **Step 4: 启动并验证骨架**

Run: `node prototypes/serve.js`（后台运行）
用 browser-use `navigate_page` 打开 `http://localhost:4173`，再执行：

```js
JSON.stringify({
  app: !!document.querySelector('.app'),
  theme: document.documentElement.dataset.theme,
  tabs: document.querySelectorAll('.tab').length,
})
```

Expected: `{"app":true,"theme":"dark","tabs":0}`（tabs 在 Task 2 渲染）

- [ ] **Step 5: Commit**

```bash
git add prototypes/serve.js prototypes/index.html prototypes/styles.css
git -c user.name="pig strong" -c user.email="29220747+pigstrong9@users.noreply.github.com" commit -m "feat(prototype): scaffold window shell and theme tokens"
```

---

### Task 2: Mock 数据、分类 Tab 与搜索

**Files:**
- Create: `prototypes/mock-data.js`
- Modify: `prototypes/app.js`（新建文件）
- Modify: `prototypes/index.html`（在 `</body>` 前引入脚本）
- Modify: `prototypes/styles.css`（追加 tab 与列表状态样式）

**Interfaces:**
- Produces:
  - `window.MOCK_ITEMS: Item[]`，`Item = { id, kind: 'text'|'image'|'files', content: string|string[], extra?: object, preview: string, size: number, favorite: boolean, pinned: boolean, copy_count: number, created_at: number, updated_at: number }`
  - `renderTabs()`、`renderList()`、`renderAll()`、`setTab(key)`、`visibleItems()`、`sortedItems()` 供后续任务复用

- [ ] **Step 1: 写 mock 数据并引入脚本**

```js
// prototypes/mock-data.js
const now = Date.now();
const min = 60 * 1000;
const day = 24 * 60 * min;

function svgImage(width, height, label) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
    <rect width="100%" height="100%" fill="#111418"/>
    <rect x="6" y="6" width="90%" height="20" rx="3" fill="#1d2530"/>
    <text x="14" y="21" font-family="Consolas, monospace" font-size="12" fill="#7dd3fc">${label}</text>
    <rect x="6" y="34" width="70%" height="10" rx="2" fill="#243244"/>
    <rect x="6" y="50" width="55%" height="10" rx="2" fill="#243244"/>
  </svg>`;
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

window.MOCK_ITEMS = [
  {
    id: 'itm-001',
    kind: 'image',
    content: 'mock://image/terminal',
    extra: { width: 1205, height: 903, thumb: svgImage(1205, 903, 'ALL_RECORDS = ...') },
    preview: '',
    size: 1205 * 903,
    favorite: false,
    pinned: false,
    copy_count: 2,
    created_at: now - 1 * min,
    updated_at: now - 1 * min,
  },
  {
    id: 'itm-002',
    kind: 'files',
    content: [
      'C:/Users/pigstrong/Documents/内部 gitlab 使用指南-v2.pdf',
      'C:/Users/pigstrong/Downloads/图片批量处理 MAC 版-1.9.1.upxs',
      'C:/Users/pigstrong/Downloads/图片批量处理-1.9.1.upxs',
      'C:/Users/pigstrong/Downloads/图片批量处理-2.0.1.upxs',
    ],
    extra: { count: 4 },
    preview: '内部 gitlab 使用指南-v2.pdf 等 4 个文件',
    size: 4,
    favorite: false,
    pinned: false,
    copy_count: 3,
    created_at: now - 3 * min,
    updated_at: now - 3 * min,
  },
  {
    id: 'itm-003',
    kind: 'text',
    content: `if (!ALL_RECORDS) return
const deletedHashes = JSON.parse(window.localStorage.getItem(event.key))
ALL_RECORDS = ALL_RECORDS.filter(x => !deletedHashes.includes(x.hash))
const tabIndex = this.state.tabIndex
if (IS_PIN_SCREEN) {
  if (tabIndex === 'clipboard') this.tabChange(tabIndex)`,
    preview: 'if (!ALL_RECORDS) return …',
    size: 440,
    favorite: false,
    pinned: false,
    copy_count: 4,
    created_at: now - 9 * min,
    updated_at: now - 9 * min,
  },
  {
    id: 'itm-004',
    kind: 'text',
    content: 'window.localStorage.setItem("records-removed", JSON.stringify(recordItems.map(x => x.hash)))',
    preview: 'window.localStorage.setItem("records-removed", …',
    size: 92,
    favorite: true,
    pinned: false,
    copy_count: 1,
    created_at: now - 10 * min,
    updated_at: now - 10 * min,
  },
  {
    id: 'itm-005',
    kind: 'text',
    content: 'this.tabChange(this.state.tabIndex)',
    preview: 'this.tabChange(this.state.tabIndex)',
    size: 35,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 23 * min,
    updated_at: now - 23 * min,
  },
  {
    id: 'itm-006',
    kind: 'text',
    content: 'ALL_RECORDS = ALL_RECORDS.filter(x => !recordItems.includes(x))',
    preview: 'ALL_RECORDS = ALL_RECORDS.filter(x => !recordItems.includes(x))',
    size: 63,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 24 * min,
    updated_at: now - 24 * min,
  },
  {
    id: 'itm-007',
    kind: 'text',
    content: 'https://github.com/pigstrong9/LoveqPodcast/releases/download/v1.4/app-release.apk',
    preview: 'https://github.com/pigstrong9/LoveqPodcast/releases/…',
    size: 81,
    favorite: true,
    pinned: false,
    copy_count: 2,
    created_at: now - 2 * day,
    updated_at: now - 2 * day,
  },
  {
    id: 'itm-008',
    kind: 'text',
    content: '#为了上下班路上听播客，我花了 2 天时间开发了一个 APP\n\n《一些事一些情》这个电台节目，我从高中就开始听了，断断续续十几年，到现在还是每天上下班开车路上的固定搭子。\n\n但说实话，现在想好好听个节目，还真没那么简单。',
    preview: '#为了上下班路上听播客，我花了 2 天时间开发了一个 APP',
    size: 688,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 35 * day,
    updated_at: now - 35 * day,
  },
  {
    id: 'itm-009',
    kind: 'image',
    content: 'mock://image/chart',
    extra: { width: 1920, height: 1080, thumb: svgImage(1920, 1080, 'chart-2026-q4.png') },
    preview: '',
    size: 1920 * 1080,
    favorite: true,
    pinned: false,
    copy_count: 1,
    created_at: now - 40 * day,
    updated_at: now - 40 * day,
  },
  {
    id: 'itm-010',
    kind: 'files',
    content: ['D:/workspace/uClipboard/prototypes/index.html', 'D:/workspace/uClipboard/prototypes/app.js'],
    extra: { count: 2 },
    preview: 'index.html 等 2 个文件',
    size: 2,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 41 * day,
    updated_at: now - 41 * day,
  },
  {
    id: 'itm-011',
    kind: 'text',
    content: 'npm install\nnpm run dev\ncargo tauri dev',
    preview: 'npm install / npm run dev / cargo tauri dev',
    size: 38,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 42 * day,
    updated_at: now - 42 * day,
  },
  {
    id: 'itm-012',
    kind: 'text',
    content: '这是一段用于测试长文本展开与收起的记录。'.repeat(20),
    preview: '这是一段用于测试长文本展开与收起的记录。'.repeat(3),
    size: 420,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 50 * day,
    updated_at: now - 50 * day,
  },
];
```

在 `prototypes/index.html` 的 `</body>` 前加入：

```html
<script src="mock-data.js"></script>
<script src="app.js"></script>
```

- [ ] **Step 2: 写图标、状态、排序、过滤与 Tab 渲染**

```js
// prototypes/app.js（节选）
const ICONS = {
  clipboard: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>',
  layers: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m12 2 9 5-9 5-9-5 9-5Z"/><path d="m3 12 9 5 9-5"/><path d="m3 17 9 5 9-5"/></svg>',
  text: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>',
  image: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>',
  files: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/></svg>',
  star: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m12 2 3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2Z"/></svg>',
  file: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/></svg>',
  pin: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16h14v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1Z"/></svg>',
  paste: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1Z"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>',
  trash: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M10 11v6"/><path d="M14 11v6"/></svg>',
  gear: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"/></svg>',
  more: '<svg viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.6"/><circle cx="12" cy="12" r="1.6"/><circle cx="19" cy="12" r="1.6"/></svg>',
  sun: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>',
  search: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/></svg>',
};

document.querySelector('.brand-logo').innerHTML = ICONS.clipboard;
document.querySelector('.search-icon').innerHTML = ICONS.search;
document.getElementById('themeBtn').innerHTML = ICONS.sun;
document.getElementById('settingsBtn').innerHTML = ICONS.gear;
document.getElementById('moreBtn').innerHTML = ICONS.more;

const state = {
  items: window.MOCK_ITEMS.map((item) => ({ ...item })),
  tab: 'all',
  query: '',
  selected: 0,
  expanded: new Set(),
};

const TABS = [
  { key: 'all', label: '全部', icon: 'layers' },
  { key: 'text', label: '文本', icon: 'text' },
  { key: 'image', label: '图像', icon: 'image' },
  { key: 'files', label: '文件', icon: 'files' },
  { key: 'favorite', label: '收藏', icon: 'star' },
];

function sortedItems() {
  if (state.tab === 'favorite') {
    return state.items
      .filter((item) => item.favorite)
      .sort((a, b) => b.updated_at - a.updated_at);
  }
  return state.items
    .slice()
    .sort((a, b) => Number(b.pinned) - Number(a.pinned) || b.updated_at - a.updated_at);
}

function visibleItems() {
  const keyword = state.query.trim().toLowerCase();
  let items = sortedItems();
  if (state.tab !== 'all' && state.tab !== 'favorite') {
    items = items.filter((item) => item.kind === state.tab);
  }
  if (keyword) {
    items = items.filter((item) => {
      const text = item.kind === 'files' ? item.content.join('\n') : item.content;
      return text.toLowerCase().includes(keyword);
    });
  }
  return items;
}

function renderTabs() {
  const counts = {
    all: state.items.length,
    text: state.items.filter((item) => item.kind === 'text').length,
    image: state.items.filter((item) => item.kind === 'image').length,
    files: state.items.filter((item) => item.kind === 'files').length,
    favorite: state.items.filter((item) => item.favorite).length,
  };
  document.getElementById('tabs').innerHTML = TABS.map(
    (tab) => `
      <button class="tab ${state.tab === tab.key ? 'active' : ''}" data-tab="${tab.key}" role="tab">
        <span class="tab-icon">${ICONS[tab.icon]}</span>
        <span>${tab.label}</span>
        ${tab.key === 'favorite' ? `<span class="tab-count">${counts.favorite}</span>` : ''}
      </button>`
  ).join('');
}

function setTab(key) {
  state.tab = key;
  state.selected = 0;
  renderAll();
}

function renderAll() {
  renderTabs();
  renderList();
}

document.getElementById('tabs').addEventListener('click', (event) => {
  const tab = event.target.closest('[data-tab]');
  if (tab) setTab(tab.dataset.tab);
});
```

```css
/* styles.css 追加 */
.tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 14px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-2);
  font-size: 14px;
  cursor: pointer;
}
.tab:hover { background: var(--hover); color: var(--text); }
.tab.active { background: var(--surface-2); color: var(--accent); }
.tab-icon svg { width: 16px; height: 16px; }
.tab-count { font-size: 12px; color: var(--text-2); }
```

- [ ] **Step 3: 写最小列表渲染与空状态**

```js
function renderList() {
  const list = document.getElementById('list');
  const items = visibleItems();
  state.selected = Math.min(state.selected, Math.max(items.length - 1, 0));
  if (!items.length) {
    list.innerHTML = `<div class="empty">
      <div class="empty-icon">${ICONS.clipboard}</div>
      <p>${state.query ? '没有匹配的记录' : '还没有剪贴板记录'}</p>
      <span>${state.query ? '换个关键词试试' : '复制任意内容后会自动出现在这里'}</span>
    </div>`;
    renderStatus();
    return;
  }
  list.innerHTML = items.map((item, index) => renderItem(item, index)).join('');
  renderStatus();
}

function renderStatus() {
  document.getElementById('statusText').textContent =
    `监控中 · 共 ${state.items.length} 条`;
}

document.getElementById('searchInput').addEventListener('input', (event) => {
  state.query = event.target.value;
  state.selected = 0;
  renderList();
});

// 最小渲染：Task 3 会替换为完整的条目卡片
function renderItem(item, index) {
  return `<article class="item ${index === state.selected ? 'selected' : ''}" data-id="${item.id}" data-index="${index}">
    <div class="item-main">${escapeHtml(item.preview)}</div>
  </article>`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

renderAll();
```

```css
.empty {
  height: 100%;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: 6px;
  color: var(--text-2);
  font-size: 13px;
}
.empty-icon svg { width: 42px; height: 42px; opacity: 0.35; }
```

- [ ] **Step 4: 验证分类与搜索**

打开页面后执行：

```js
const tabs = [...document.querySelectorAll('.tab')].map((el) => el.dataset.tab);
const counts = { all: document.querySelectorAll('.item').length };
document.querySelector('[data-tab="favorite"]').click();
counts.favorite = document.querySelectorAll('.item').length;
document.querySelector('[data-tab="all"]').click();
const search = document.getElementById('searchInput');
search.value = 'gitlab';
search.dispatchEvent(new Event('input'));
JSON.stringify({ tabs, counts, searched: document.querySelectorAll('.item').length });
```

Expected: `tabs` 为 `["all","text","image","files","favorite"]`，`counts.all` 为 12，`counts.favorite` 为 3，`searched` 为 1。

- [ ] **Step 5: Commit**

```bash
git add prototypes/mock-data.js prototypes/app.js prototypes/styles.css
git -c user.name="pig strong" -c user.email="29220747+pigstrong9@users.noreply.github.com" commit -m "feat(prototype): add mock data, tabs and search"
```

---

### Task 3: 条目卡片与元信息

**Files:**
- Modify: `prototypes/app.js`（`renderItem`、`formatRelativeTime`、展开/收起）
- Modify: `prototypes/styles.css`（条目样式）

**Interfaces:**
- Produces: `.item[data-id]`、`.item.selected`；文本 4 行截断 + `.expand-btn`；图片缩略图；文件列表最多 4 行 + 汇总；右下序号（仅前 9 条）

- [ ] **Step 1: 用完整渲染替换 Task 2 的最小 renderItem**

`escapeHtml` 已在 Task 2 定义，这里直接使用。

```js
function formatRelativeTime(ts) {
  const diff = Date.now() - ts;
  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < minute) return '刚用';
  if (diff < hour) return `${Math.floor(diff / minute)} 分钟前`;
  if (diff < day) return `${Math.floor(diff / hour)} 小时前`;
  return `${Math.floor(diff / day)} 天前`;
}

function renderItem(item, index) {
  const meta =
    item.kind === 'text'
      ? `${item.size} 字符`
      : item.kind === 'image'
        ? `${item.extra.width} × ${item.extra.height}`
        : `${item.extra.count} 个文件`;

  let body = '';
  if (item.kind === 'text') {
    const expanded = state.expanded.has(item.id);
    body = `<div class="text-preview ${expanded ? 'expanded' : ''}">${escapeHtml(item.content)}</div>
      ${item.content.length > 180 ? `<button class="expand-btn" data-expand="${item.id}">${expanded ? '收起' : '展开'}</button>` : ''}`;
  } else if (item.kind === 'image') {
    body = `<div class="image-preview"><img src="${item.extra.thumb}" alt="剪贴板图片" /></div>`;
  } else {
    const files = item.content.slice(0, 4);
    body = `<ul class="file-list">${files
      .map((file) => {
        const name = file.split(/[\\/]/).pop();
        return `<li><span class="file-icon">${ICONS.file}</span><span class="file-name">${escapeHtml(name)}</span></li>`;
      })
      .join('')}</ul>`;
  }

  return `<article class="item ${index === state.selected ? 'selected' : ''}" data-id="${item.id}" data-index="${index}">
    <div class="item-main">${body}</div>
    <div class="item-meta">
      <span class="item-time">${formatRelativeTime(item.updated_at)}</span>
      <span class="item-size">${meta}</span>
      ${index < 9 ? `<span class="item-index">${index + 1}</span>` : ''}
    </div>
    <div class="row-actions">
      <button class="icon-btn" data-action="paste" title="粘贴">${ICONS.paste}</button>
      <button class="icon-btn" data-action="favorite" title="收藏">${ICONS.star}</button>
      <button class="icon-btn" data-action="delete" title="删除">${ICONS.trash}</button>
    </div>
    ${item.pinned ? `<span class="pin-badge">${ICONS.pin}</span>` : ''}
  </article>`;
}
```

同一文件追加展开/收起监听（Task 4 会替换成包含选中、行内操作的完整监听）：

```js
document.getElementById('list').addEventListener('click', (event) => {
  const btn = event.target.closest('[data-expand]');
  if (!btn) return;
  const id = btn.dataset.expand;
  state.expanded.has(id) ? state.expanded.delete(id) : state.expanded.add(id);
  renderList();
});
```

- [ ] **Step 2: 写条目样式**

```css
/* styles.css 追加 */
.item {
  position: relative;
  padding: 12px 14px 10px;
  margin-bottom: 6px;
  border-radius: var(--radius-card);
  background: var(--surface);
  border: 1px solid transparent;
  cursor: default;
}
.item:hover { background: var(--surface-2); }
.item.selected { border-color: var(--selected); }

.item-main { color: var(--text); font-size: 14px; line-height: 1.55; }
.text-preview {
  font-family: var(--font-mono);
  white-space: pre-wrap;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
}
.text-preview.expanded { display: block; -webkit-line-clamp: unset; }
.expand-btn {
  margin-top: 4px;
  border: 0;
  background: transparent;
  color: var(--text-2);
  font-size: 12px;
  cursor: pointer;
}
.expand-btn:hover { color: var(--accent); }

.image-preview { display: grid; place-items: center; padding: 6px 0; }
.image-preview img {
  max-width: 70%;
  max-height: 110px;
  border-radius: 6px;
  border: 1px solid var(--border);
}

.file-list { list-style: none; margin: 0; padding: 0; display: grid; gap: 4px; }
.file-list li { display: flex; align-items: center; gap: 8px; min-width: 0; }
.file-icon svg { width: 16px; height: 16px; color: var(--text-2); }
.file-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.item-meta {
  margin-top: 6px;
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-2);
  font-size: 12px;
}
.item-size { margin-left: auto; }
.item-index { width: 18px; text-align: right; }

.pin-badge {
  position: absolute;
  top: 10px;
  right: 12px;
  color: var(--accent);
}
.pin-badge svg { width: 14px; height: 14px; }
```

- [ ] **Step 3: 验证条目渲染**

```js
const items = [...document.querySelectorAll('.item')];
JSON.stringify({
  count: items.length,
  kinds: items.map((el) => el.querySelector('.image-preview, .file-list, .text-preview')?.className ?? 'unknown'),
  indexes: items.slice(0, 3).map((el) => el.querySelector('.item-index')?.textContent ?? ''),
})
```

Expected: `count` 为 12；`kinds` 中同时出现 `image-preview`、`file-list`、`text-preview`；前 3 条序号为 `["1","2","3"]`。

- [ ] **Step 4: 验证展开/收起**

点击第一条长文本（`itm-012` 需先切换到「文本」并滚到底部）的展开按钮，检查 `-webkit-line-clamp` 从 `4` 变为可见全文（`display:block`）。用 `evaluate_script` 断言 `.text-preview.expanded` 存在。

- [ ] **Step 5: Commit**

```bash
git add prototypes/app.js prototypes/styles.css
git -c user.name="pig strong" -c user.email="29220747+pigstrong9@users.noreply.github.com" commit -m "feat(prototype): render text, image and file items"
```

---

### Task 4: 键盘、鼠标与「使用即置顶」

**Files:**
- Modify: `prototypes/app.js`（选择、动作、快捷键、点击事件）
- Modify: `prototypes/styles.css`（悬停操作按钮）

**Interfaces:**
- Produces: `select(index)`、`pasteItem(id)`、`copyItem(id)`、`toggleFavorite(id)`、`togglePin(id)`、`removeItem(id)`、`touchItem(id)`；列表项 `data-id` 可被点击定位

- [ ] **Step 1: 写动作与置顶逻辑**

```js
function findItem(id) {
  return state.items.find((item) => item.id === id);
}

function touchItem(id) {
  const item = findItem(id);
  if (!item) return;
  item.updated_at = Date.now();
  item.copy_count += 1;
}

let toastTimer = 0;

function toast(message) {
  const el = document.getElementById('toast');
  el.textContent = message;
  el.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (el.hidden = true), 1600);
}

function hideWindow() {
  document.getElementById('app').classList.add('is-hidden');
  document.getElementById('restoreHint').hidden = false;
}

document.getElementById('restoreHint').addEventListener('click', () => {
  document.getElementById('app').classList.remove('is-hidden');
  document.getElementById('restoreHint').hidden = true;
  state.query = '';
  document.getElementById('searchInput').value = '';
  state.selected = 0;
  renderAll();
});

function pasteItem(id) {
  touchItem(id);
  renderAll();
  toast('已粘贴（原型）');
  hideWindow();
}

function copyItem(id) {
  touchItem(id);
  toast('已复制');
  renderAll();
}

function toggleFavorite(id) {
  const item = findItem(id);
  if (!item) return;
  item.favorite = !item.favorite;
  renderAll();
}

function togglePin(id) {
  const item = findItem(id);
  if (!item) return;
  item.pinned = !item.pinned;
  renderAll();
}

function removeItem(id) {
  state.items = state.items.filter((item) => item.id !== id);
  state.selected = 0;
  renderAll();
  toast('已删除');
}

function currentItem() {
  return visibleItems()[state.selected] ?? null;
}

function select(index) {
  state.selected = index;
  renderList();
  const el = document.querySelector('.item.selected');
  el?.scrollIntoView({ block: 'nearest' });
}
```

- [ ] **Step 2: 写键盘处理**

```js
document.addEventListener('keydown', (event) => {
  const target = event.target;
  const inSearch = target instanceof HTMLElement && target.id === 'searchInput';
  const meta = event.metaKey || event.ctrlKey;

  if (event.key === 'Escape') {
    if (!document.getElementById('modal').hidden) return closeModal();
    if (!document.getElementById('contextMenu').hidden) return closeContextMenu();
    if (state.query) {
      state.query = '';
      document.getElementById('searchInput').value = '';
      renderList();
      return;
    }
    if (inSearch) return target.blur();
    return hideWindow();
  }

  if (inSearch) return;

  const items = visibleItems();
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    return select(Math.min(state.selected + 1, items.length - 1));
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault();
    return select(Math.max(state.selected - 1, 0));
  }
  if (event.key === 'Tab') {
    event.preventDefault();
    const delta = event.shiftKey ? -1 : 1;
    return select(Math.min(Math.max(state.selected + delta, 0), items.length - 1));
  }
  if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
    event.preventDefault();
    const index = TABS.findIndex((tab) => tab.key === state.tab);
    const next = (index + (event.key === 'ArrowRight' ? 1 : -1) + TABS.length) % TABS.length;
    setTab(TABS[next].key);
    return;
  }
  if (event.key === 'Enter') {
    const item = currentItem();
    if (item) pasteItem(item.id);
    return;
  }
  if (event.key === ' ') {
    event.preventDefault();
    const item = currentItem();
    if (item && typeof openPreview === 'function') openPreview(item);
    return;
  }
  if (event.key === 'Delete' || (event.metaKey && event.key === 'Backspace')) {
    const item = currentItem();
    if (item) removeItem(item.id);
    return;
  }
  if (meta && event.key.toLowerCase() === 'c') {
    event.preventDefault();
    const item = currentItem();
    if (item) copyItem(item.id);
    return;
  }
  if (meta && event.key.toLowerCase() === 'd') {
    event.preventDefault();
    const item = currentItem();
    if (item) toggleFavorite(item.id);
    return;
  }
  if (meta && event.key.toLowerCase() === 'p') {
    event.preventDefault();
    const item = currentItem();
    if (item) togglePin(item.id);
    return;
  }
  if (!meta && !event.altKey && /^[1-9]$/.test(event.key)) {
    const item = items[Number(event.key) - 1];
    if (item) pasteItem(item.id);
    return;
  }
  if (!meta && !event.altKey && (event.key === '/' || /^[a-zA-Z]$/.test(event.key))) {
    event.preventDefault();
    const search = document.getElementById('searchInput');
    search.focus();
    if (event.key !== '/') search.value += event.key;
    state.query = search.value;
    renderList();
  }
});
```

- [ ] **Step 3: 替换为完整的列表点击监听**

分类点击、搜索聚焦和 `setTab` 已在 Task 2 实现。这里把 Task 3 的展开监听替换为同时处理行内操作、展开与选中的完整版本。

```js
document.getElementById('list').addEventListener('click', (event) => {
  const actionBtn = event.target.closest('[data-action]');
  if (actionBtn) {
    const id = actionBtn.closest('.item').dataset.id;
    const action = actionBtn.dataset.action;
    if (action === 'paste') return pasteItem(id);
    if (action === 'favorite') return toggleFavorite(id);
    if (action === 'delete') return removeItem(id);
  }
  const item = event.target.closest('.item');
  if (!item) return;
  if (event.target.closest('[data-expand]')) {
    const id = event.target.closest('[data-expand]').dataset.expand;
    state.expanded.has(id) ? state.expanded.delete(id) : state.expanded.add(id);
    return renderList();
  }
  select(Number(item.dataset.index));
});

document.getElementById('list').addEventListener('dblclick', (event) => {
  const item = event.target.closest('.item');
  if (item) pasteItem(item.dataset.id);
});
```

```css
.item .row-actions {
  position: absolute;
  right: 44px;
  top: 10px;
  display: none;
  gap: 2px;
}
.item:hover .row-actions { display: flex; }
```

- [ ] **Step 4: 验证「使用即置顶」**

```js
// 先回到全部、清空搜索，选中第 3 条并回车
state.tab = 'all'; state.query = ''; state.selected = 2;
document.getElementById('searchInput').value = '';
renderAll();
const before = [...document.querySelectorAll('.item')].map((el) => el.dataset.id);
document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
const after = [...document.querySelectorAll('.item')].map((el) => el.dataset.id);
JSON.stringify({ before: before.slice(0, 4), after: after.slice(0, 4) });
```

Expected: `after[0] === before[2]`，且列表重新打开时该条位于第一位。再验证外部重复内容只移动不新增：随后检查 `state.items.length` 仍为 12。

- [ ] **Step 5: 验证其它快捷键**

- `ArrowDown`：`.item.selected` 的 `data-index` 加 1
- `1`：第一条被粘贴并触发置顶（toast 出现）；随后点击恢复提示继续
- `Ctrl+D`：当前条目收藏状态变化，收藏 Tab 数量变化
- `Delete`：条目数减 1
- `/`：搜索框获得焦点

- [ ] **Step 6: Commit**

```bash
git add prototypes/app.js prototypes/styles.css
git -c user.name="pig strong" -c user.email="29220747+pigstrong9@users.noreply.github.com" commit -m "feat(prototype): add keyboard, mouse and use-to-top behavior"
```

---

### Task 5: 预览弹窗与右键菜单

**Files:**
- Modify: `prototypes/app.js`（`openPreview`、`closeModal`、`openContextMenu`、`closeContextMenu`）
- Modify: `prototypes/styles.css`（弹窗、菜单样式）

**Interfaces:**
- Produces: `openPreview(item)`、`closeModal()`、`openContextMenu(event, item)`、`closeContextMenu()`
- Consumes: Task 4 的 `toast()`、`hideWindow()`

- [ ] **Step 1: 实现预览弹窗**

```js
function openPreview(item) {
  const modal = document.getElementById('modal');
  if (item.kind === 'image') {
    modal.innerHTML = `<figure class="preview-image">
      <img src="${item.extra.thumb}" alt="剪贴板图片" />
      <figcaption>${item.extra.width} × ${item.extra.height}</figcaption>
    </figure>`;
  } else if (item.kind === 'files') {
    modal.innerHTML = `<div class="preview-text"><h3>${item.extra.count} 个文件</h3><pre>${escapeHtml(item.content.join('\n'))}</pre></div>`;
  } else {
    modal.innerHTML = `<div class="preview-text"><pre>${escapeHtml(item.content)}</pre></div>`;
  }
  modal.hidden = false;
}

function closeModal() {
  document.getElementById('modal').hidden = true;
}

document.getElementById('modal').addEventListener('click', (event) => {
  if (event.target.id === 'modal') closeModal();
});
```

- [ ] **Step 2: 实现右键菜单**

```js
function openContextMenu(event, item) {
  event.preventDefault();
  const menu = document.getElementById('contextMenu');
  menu.innerHTML = `
    <button data-action="paste">粘贴</button>
    <button data-action="copy">复制</button>
    <button data-action="favorite">${item.favorite ? '取消收藏' : '收藏'}</button>
    <button data-action="pin">${item.pinned ? '取消置顶' : '置顶'}</button>
    <hr />
    <button data-action="delete" class="danger">删除</button>`;
  menu.dataset.id = item.id;
  menu.style.left = `${Math.min(event.clientX, window.innerWidth - 160)}px`;
  menu.style.top = `${Math.min(event.clientY, window.innerHeight - 200)}px`;
  menu.hidden = false;
}

function closeContextMenu() {
  document.getElementById('contextMenu').hidden = true;
  const menu = document.getElementById('contextMenu');
  menu.onclick = null;
}

document.getElementById('list').addEventListener('contextmenu', (event) => {
  const el = event.target.closest('.item');
  if (!el) return;
  const item = findItem(el.dataset.id);
  select(Number(el.dataset.index));
  openContextMenu(event, item);
});

document.getElementById('contextMenu').addEventListener('click', (event) => {
  const button = event.target.closest('[data-action]');
  if (!button) return;
  const id = document.getElementById('contextMenu').dataset.id;
  const action = button.dataset.action;
  closeContextMenu();
  if (action === 'paste') pasteItem(id);
  if (action === 'copy') copyItem(id);
  if (action === 'favorite') toggleFavorite(id);
  if (action === 'pin') togglePin(id);
  if (action === 'delete') removeItem(id);
});

document.addEventListener('click', (event) => {
  if (!event.target.closest('#contextMenu')) closeContextMenu();
});
```

- [ ] **Step 3: 写弹窗与菜单样式**

```css
/* styles.css 追加 */
.modal-backdrop {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.55);
  z-index: 80;
}
.modal-backdrop[hidden] { display: none; }

.preview-image { margin: 0; display: grid; justify-items: center; gap: 10px; }
.preview-image img {
  max-width: min(760px, 86vw);
  max-height: 70vh;
  border-radius: 8px;
  border: 1px solid var(--border);
}
.preview-image figcaption { color: var(--text-2); font-size: 12px; }

.preview-text {
  width: min(720px, 86vw);
  max-height: 70vh;
  overflow: auto;
  padding: 20px;
  border-radius: 10px;
  background: var(--surface);
  border: 1px solid var(--border);
}
.preview-text pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.6;
}

.context-menu {
  position: fixed;
  z-index: 85;
  min-width: 140px;
  padding: 6px;
  border-radius: 8px;
  background: var(--surface);
  border: 1px solid var(--border);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  display: grid;
}
.context-menu[hidden] { display: none; }
.context-menu button {
  padding: 7px 10px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  text-align: left;
  font-size: 13px;
  cursor: pointer;
}
.context-menu button:hover { background: var(--hover); }
.context-menu button.danger { color: #f87171; }
.context-menu hr { width: 100%; border: 0; border-top: 1px solid var(--border); margin: 4px 0; }

.is-hidden { display: none; }
.restore-hint {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  color: #9aa0a6;
  font-size: 14px;
  cursor: pointer;
}
.restore-hint[hidden] { display: none; }
```

- [ ] **Step 4: 验证弹窗与菜单**

- 选中图片条目按 `Space`：`#modal` 不再 hidden，包含 `img`
- `Escape`：`#modal` 回到 hidden
- 右键第二条：`#contextMenu` 可见，点击「收藏」后收藏计数 +1
- 双击条目：toast 可见且内容为「已粘贴（原型）」

- [ ] **Step 5: Commit**

```bash
git add prototypes/app.js prototypes/styles.css
git -c user.name="pig strong" -c user.email="29220747+pigstrong9@users.noreply.github.com" commit -m "feat(prototype): add preview modal and context menu"
```

---

### Task 6: 设置页与主题切换

**Files:**
- Modify: `prototypes/app.js`（设置面板、主题切换、清空确认）
- Modify: `prototypes/styles.css`（设置布局）
- Modify: `prototypes/index.html`（设置容器）

**Interfaces:**
- Produces: `openSettings()`、`closeSettings()`、`setTheme(theme)`；设置面板覆盖主窗口，`Escape` 关闭

- [ ] **Step 1: 增加设置容器**

在 `.app` 内、`.statusbar` 前加入：

```html
<div class="settings" id="settings" hidden>
  <aside class="settings-nav">
    <button class="active" data-section="basic">基本</button>
    <button data-section="history">历史</button>
    <button data-section="privacy">隐私</button>
    <button data-section="about">关于</button>
  </aside>
  <section class="settings-body" id="settingsBody"></section>
</div>
```

- [ ] **Step 2: 渲染设置分区**

```js
const SETTINGS_SECTIONS = {
  basic: [
    ['全局快捷键', 'Alt + V'],
    ['开机自启', '已开启'],
    ['失焦自动隐藏', '已开启'],
    ['粘贴为纯文本', '已关闭'],
    ['主题', '跟随系统'],
    ['语言', '简体中文'],
  ],
  history: [
    ['历史上限', '500 条'],
    ['保留天数', '30 天'],
    ['记录文本', '开启'],
    ['记录图片', '开启'],
    ['记录文件', '开启'],
    ['最短文本长度', '3 字符'],
  ],
  privacy: [
    ['忽略应用', 'KeePass、1Password、Bitwarden'],
    ['隐私剪贴板格式', '已启用识别'],
    ['数据存储', '仅本机 SQLite 与本地图片文件'],
  ],
  about: [
    ['版本', '0.1.0 原型'],
    ['许可证', 'Apache-2.0'],
    ['检查更新', '通过 GitHub Releases'],
  ],
};

function renderSettings(section = 'basic') {
  document.getElementById('settingsBody').innerHTML = `
    <h2>${section === 'basic' ? '基本设置' : section === 'history' ? '历史记录' : section === 'privacy' ? '隐私' : '关于'}</h2>
    <ul class="setting-list">
      ${SETTINGS_SECTIONS[section]
        .map(([label, value]) => `<li><span>${label}</span><strong>${value}</strong></li>`)
        .join('')}
    </ul>
    ${section === 'basic' ? `<div class="theme-row">
      <span>界面主题</span>
      <select id="themeSelect">
        <option value="system">跟随系统</option>
        <option value="light">浅色</option>
        <option value="dark">深色</option>
      </select>
    </div>` : ''}`;
}

function openSettings() {
  document.getElementById('settings').hidden = false;
  renderSettings('basic');
}

function closeSettings() {
  document.getElementById('settings').hidden = true;
}

document.getElementById('settingsBtn').addEventListener('click', openSettings);
document.getElementById('settingsTextBtn').addEventListener('click', openSettings);
document.getElementById('settings').addEventListener('click', (event) => {
  const nav = event.target.closest('[data-section]');
  if (nav) {
    document.querySelectorAll('.settings-nav button').forEach((btn) => btn.classList.remove('active'));
    nav.classList.add('active');
    renderSettings(nav.dataset.section);
  }
});

document.getElementById('settingsBody').addEventListener('change', (event) => {
  if (event.target.id === 'themeSelect') setTheme(event.target.value);
});
```

- [ ] **Step 3: 主题切换与清空**

```js
function setTheme(theme) {
  const resolved =
    theme === 'system'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
      : theme;
  document.documentElement.dataset.theme = resolved;
}

document.getElementById('themeBtn').addEventListener('click', () => {
  const next = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
  setTheme(next);
});

document.getElementById('clearBtn').addEventListener('click', () => {
  if (!window.confirm('确定清空全部剪贴板历史？')) return;
  state.items = [];
  state.selected = 0;
  renderAll();
  toast('历史已清空');
});
```

在全局 `keydown` 的 `Escape` 分支最前面补充（`closeSettings` 为函数声明，提升后可用）：

```js
if (!document.getElementById('settings').hidden) return closeSettings();
```

- [ ] **Step 4: 写设置样式**

```css
.settings {
  position: absolute;
  inset: 56px 0 44px;
  display: grid;
  grid-template-columns: 150px 1fr;
  background: var(--bg);
  z-index: 70;
}
.settings[hidden] { display: none; }
.settings-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px 10px;
  border-right: 1px solid var(--border);
}
.settings-nav button {
  padding: 8px 10px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-2);
  text-align: left;
  font-size: 13px;
  cursor: pointer;
}
.settings-nav button.active { background: var(--surface-2); color: var(--accent); }
.settings-body { padding: 20px 24px; overflow: auto; }
.settings-body h2 { margin: 0 0 16px; font-size: 16px; }
.setting-list { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; }
.setting-list li {
  display: flex;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--surface);
  font-size: 13px;
}
.setting-list strong { color: var(--text-2); font-weight: 500; }
.theme-row {
  margin-top: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--surface);
  font-size: 13px;
}
```

注意：`.app` 的 `position: relative` 已在 Task 1 设置，无需重复添加。

- [ ] **Step 5: 验证设置与主题**

- 点击齿轮：设置面板出现，四个分区可切换
- 点击主题按钮：`data-theme` 在 `light`/`dark` 间切换，颜色 token 生效
- `Escape`：设置面板关闭
- 点击清空并确认：列表变为空状态

- [ ] **Step 6: Commit**

```bash
git add prototypes/index.html prototypes/app.js prototypes/styles.css
git -c user.name="pig strong" -c user.email="29220747+pigstrong9@users.noreply.github.com" commit -m "feat(prototype): add settings panel and theme toggle"
```

---

### Task 7: 视觉校准与 M0 验收

**Files:**
- Modify: `prototypes/styles.css`（按对比结果微调间距、字号、颜色）
- Create: `docs/superpowers/plans/2026-11-20-uclipboard-m0-acceptance.md`（验收记录）

**Interfaces:**
- Produces: 可交给用户并排对比的最终原型；验收清单

- [ ] **Step 1: 启动并截取双主题截图**

- `node prototypes/serve.js`
- browser-use 设置视口 880×660，分别截图：
  - 暗色：`data-theme="dark"`，停留在「全部」
  - 亮色：`data-theme="light"`
  - 收藏页
  - 设置页

- [ ] **Step 2: 与参考对照并微调**

对照用户提供的截图与官方介绍页截图，逐项检查：

- 头部高度、搜索框宽度与圆角
- 分类 Tab 的间距、激活态颜色、收藏计数
- 条目内边距、字号、行高、预览最大行数
- 相对时间/字符数/序号的位置
- 选中描边、悬停背景
- 图片缩略图高度、文件列表行高
- 亮暗两套颜色的对比度与主色

发现偏差直接改 `styles.css`，每次改动后重新截图。

- [ ] **Step 3: 执行验收清单**

```js
JSON.stringify({
  tabs: [...document.querySelectorAll('.tab')].map((el) => el.textContent.trim()),
  itemCount: document.querySelectorAll('.item').length,
  hasText: !!document.querySelector('.text-preview'),
  hasImage: !!document.querySelector('.image-preview'),
  hasFiles: !!document.querySelector('.file-list'),
  theme: document.documentElement.dataset.theme,
})
```

Expected: 5 个 Tab、12 条记录、三类条目都存在。

- [ ] **Step 4: 验证键盘全流程**

按顺序执行并确认：

1. `ArrowDown` ×3 → 选中第 4 条
2. `Enter` → 第 4 条移动到第一位
3. 重新打开（点击恢复提示）→ 第一条为刚用过的记录
4. `Ctrl+D` → 收藏计数变化
5. `1` → 第一条粘贴置顶
6. `Space` → 预览打开，`Escape` 关闭
7. `/` → 搜索框聚焦
8. `Esc` → 清空搜索 → 再次 `Esc` 隐藏窗口

- [ ] **Step 5: 写验收记录并提交**

在 `docs/superpowers/plans/2026-11-20-uclipboard-m0-acceptance.md` 记录：截图路径、对照结论、待用户确认项、与 spec 的偏差。

```bash
git add prototypes docs/superpowers/plans/2026-11-20-uclipboard-m0-acceptance.md
git -c user.name="pig strong" -c user.email="29220747+pigstrong9@users.noreply.github.com" commit -m "feat(prototype): polish visuals and record M0 acceptance"
```

---

## 后续计划

M0 经用户确认后，另写 `docs/superpowers/plans/*-m1-core.md` 覆盖 Tauri 骨架、SQLite、轮询采集与列表；再依次拆分 M2、M3、M4。每份计划都以本 spec 为依据，且各自产出可独立验收的软件。

## Self-Review

- Spec 覆盖：M0 验收只覆盖 spec 第 2.3、7.1、7.2、7.3 与里程碑 M0；其余功能分别属于 M1-M4，已在「后续计划」声明，不属于本计划缺失
- 占位符：无 TBD/TODO；所有任务均给出可执行代码或验证断言
- 类型一致：`Item` 字段、`visibleItems()`/`sortedItems()`/`touchItem()`/`pasteItem()` 等命名在任务间保持一致
- 已知取舍：原型不做 localStorage 持久化、不做多选/悬浮面板/拖拽，均与 spec 的 v1 非目标一致
