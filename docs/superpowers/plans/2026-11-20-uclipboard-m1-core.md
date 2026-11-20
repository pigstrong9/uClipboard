# uClipboard M1 核心版 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 M0 静态原型落成可运行的 Tauri 2 + React + Rust 应用：真实监听本机剪贴板，SQLite 持久化，完成列表、搜索、收藏、置顶、删除、复制。

**Architecture:** Tauri 2 单窗口应用。Rust 后端负责剪贴板轮询（clipboard-rs）、SQLite（rusqlite bundled）、图片落盘与缩略图、IPC 命令。React 前端复用 M0 的简约视觉，通过 `invoke` 拉取数据并监听剪贴板事件。全局快捷键、自动粘贴、托盘、临时中转独立窗口属于 M2；主题/i18n/更新/设置持久化属于 M3。

**Tech Stack:** Tauri 2、React 19 + TypeScript + Vite、Zustand、Rust、rusqlite、clipboard-rs、sha2、uuid、image。

**Spec:** `docs/superpowers/specs/2026-11-20-uclipboard-design.md`（M1 验收对应里程碑表 M1）

## Global Constraints

- 视觉沿用 M0 的简约样式：左上角「剪贴板」、蓝色主色、右侧独立按钮栏、滚动条在列表右缘
- M1 不做全局快捷键、自动粘贴、托盘、开机自启、临时中转独立窗口、自动更新；这些属于 M2/M3
- M1 的 Enter/双击行为为「复制到剪贴板」，自动粘贴在 M2 接入
- 数据只存本机：`appData/uclipboard.db`，图片在 `appData/images/`
- 无网络请求
- 匿名示例数据仅用于原型；真实运行不再写入 mock
- 提交身份使用一次性参数，不修改 git config

## File Structure

```
package.json / vite.config.ts / tsconfig.json / index.html
src/
├─ main.tsx / App.tsx
├─ components/ Titlebar.tsx, Tabs.tsx, ClipboardList.tsx, ClipboardItem.tsx, SideRail.tsx, PreviewModal.tsx, EmptyState.tsx
├─ stores/clipboard.ts
├─ lib/ipc.ts, types.ts
└─ styles.css
src-tauri/
├─ Cargo.toml / build.rs / tauri.conf.json / capabilities/default.json
├─ icons/
└─ src/
   ├─ main.rs / lib.rs
   ├─ models.rs
   ├─ storage.rs
   ├─ clipboard/{mod.rs, monitor.rs, source.rs}
   ├─ media.rs
   ├─ state.rs
   └─ commands.rs
docs/superpowers/plans/...（内部）
```

---

### Task 1: Tauri + React + TS 骨架

**Files:** `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.tsx`, `src/App.tsx`, `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

**Interfaces:**
- 前端 `npm run dev` 在 5173 提供页面；`npm run tauri dev` 启动桌面窗口
- Rust 暴露 `ping() -> String` 供冒烟

- [ ] **Step 1: 初始化 npm 项目**

```bash
npm init -y
npm i react react-dom zustand @tauri-apps/api
npm i -D vite @vitejs/plugin-react typescript @types/react @types/react-dom @tauri-apps/cli
```

`package.json` scripts：

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  }
}
```

- [ ] **Step 2: 写 Vite/TS 配置与入口**

`vite.config.ts`：

```ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
});
```

`tsconfig.json` 使用 `"jsx": "react-jsx"`、`"strict": true`、`"moduleResolution": "bundler"`。
`index.html` 挂载 `#root`，引入 `/src/main.tsx`。
`src/main.tsx` 渲染 `<App />`，引入 `styles.css`。

- [ ] **Step 3: 写 Tauri 配置与 Rust 入口**

`src-tauri/tauri.conf.json` 关键字段：

```json
{
  "productName": "uClipboard",
  "identifier": "com.uclipboard.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "uClipboard",
        "width": 880,
        "height": 660,
        "minWidth": 720,
        "minHeight": 520,
        "decorations": false,
        "center": true,
        "resizable": true
      }
    ],
    "security": { "csp": null }
  },
  "bundle": { "active": true, "targets": "all", "icon": ["icons/icon.png"] }
}
```

`src-tauri/Cargo.toml` 依赖：

```toml
[package]
name = "uclipboard"
version = "0.1.0"
edition = "2021"

[lib]
name = "uclipboard_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
clipboard-rs = "0.3"
sha2 = "0.10"
uuid = { version = "1", features = ["v7"] }
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
base64 = "0.22"
```

`lib.rs` 暴露 `ping` 命令并 `run()`；`main.rs` 调 `uclipboard_lib::run()`。

- [ ] **Step 4: 生成图标并验证**

用 `npx tauri icon <svg>` 生成 `src-tauri/icons`。运行：

```bash
npm run build
cd src-tauri && cargo check
```

Expected：TS/Vite 构建通过，`cargo check` 通过。

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json vite.config.ts tsconfig.json index.html src src-tauri
git -c user.name="Your Name" -c user.email="you@example.com" commit -m "feat: scaffold Tauri 2 + React app"
```

---

### Task 2: SQLite 存储与测试

**Files:** `src-tauri/src/models.rs`, `src-tauri/src/storage.rs`

**Interfaces:**
- `ClipKind`：`Text | Image | Files`（serde 小写）
- `ClipItem` 序列化字段与前端 `types.ts` 一致：`id, kind, content, extra, preview, size, source_app, favorite, pinned, copy_count, created_at, updated_at`
- `Store::insert(NewItem) -> ClipItem`、`Store::list(tab, query) -> Vec<ClipItem>`、`toggle_favorite/pin`、`delete`、`clear`、`touch`

- [ ] **Step 1: 建 models**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipKind { Text, Image, Files }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipItem {
    pub id: String,
    pub kind: ClipKind,
    pub content: String,
    pub extra: Option<String>,
    pub preview: String,
    pub size: i64,
    pub source_app: Option<String>,
    pub favorite: bool,
    pub pinned: bool,
    pub copy_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
```

`NewItem` 同字段去掉 id/复制计数/时间，由存储层生成。

- [ ] **Step 2: 建表与 CRUD**

按 spec 第 5.1 节建 `items` 表；`insert` 用 SHA-256 命中已有行时 `updated_at=now, copy_count+1` 并返回旧行；`list` 按 `pinned DESC, updated_at DESC`，收藏分类只看 `favorite=1` 并按 `updated_at DESC`；搜索用 `LIKE`（转义 `%` `_`）。

- [ ] **Step 3: 单测**

至少覆盖：

```rust
#[test] fn insert_then_list_orders_by_updated_at();
#[test] fn same_hash_moves_to_top_without_duplicate();
#[test] fn favorites_filter_and_pin_order();
#[test] fn delete_and_clear();
```

Run: `cd src-tauri && cargo test storage -v`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/storage.rs
git -c user.name="Your Name" -c user.email="you@example.com" commit -m "feat: add SQLite clipboard storage"
```

---

### Task 3: 剪贴板轮询与图片落盘

**Files:** `src-tauri/src/clipboard/source.rs`, `monitor.rs`, `mod.rs`, `src-tauri/src/media.rs`

**Interfaces:**
- `trait ClipboardSource { fn read(&self) -> Option<RawCapture>; fn write(&self, item: &ClipItem) -> bool; }`
- `RawCapture { kind, text, html, image_png: Option<Vec<u8>>, width, height, files: Vec<String> }`
- `spawn_monitor(app: AppHandle, store: Arc<Mutex<Store>>, suppress: Arc<Mutex<Option<String>>>)`
- 事件名：`clipboard-changed`

- [ ] **Step 1: 定义 source trait 与系统实现**

系统实现用 clipboard-rs 读取文本、文件列表、图片；读取失败返回 `None`，不 panic。写入用于 `copy_item`，写入后把内容 hash 放入 suppression，防重复入库。

- [ ] **Step 2: 归一化与哈希**

优先级 files > image > text；文本 hash = SHA-256(text)，图片 hash = SHA-256(png bytes)，文件 hash = SHA-256(排序后路径拼接)。文本短于 3 字符、超大文本/图片丢弃（上限 5MB/20MB）。

- [ ] **Step 3: 图片落盘与缩略图**

`media.rs`：原图写 `appData/images/{hash}.png`；用 `image` crate 生成宽 300 的 JPEG 缩略图 `{hash}_thumb.jpg`；列表返回 `thumb_data_url`（base64）供前端展示。文件不存在时返回空串。

- [ ] **Step 4: 轮询线程**

每 300ms 读取；与上次 hash 相同则跳过；命中 suppression 则清除并跳过；否则入库并通过 `app.emit("clipboard-changed", ())` 通知前端。线程在 Tauri setup 中启动。

- [ ] **Step 5: 单测**

用 mock `ClipboardSource` 验证：相同内容不重复、suppression 生效、三类内容归一化。

Run: `cd src-tauri && cargo test clipboard -v`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/clipboard src-tauri/src/media.rs
git -c user.name="Your Name" -c user.email="you@example.com" commit -m "feat: poll clipboard and store image thumbnails"
```

---

### Task 4: Tauri 命令、状态与启动接线

**Files:** `src-tauri/src/state.rs`, `commands.rs`, `lib.rs`, `src-tauri/tauri.conf.json`（如需要）

**Interfaces:**
- `AppState { store: Mutex<Store>, suppress: Mutex<Option<String>> }`
- 命令：`list_items(tab, query)`, `toggle_favorite(id)`, `toggle_pin(id)`, `delete_item(id)`, `clear_items()`, `copy_item(id)`, `stats()`

- [ ] **Step 1: 初始化数据库**

在 `setup` 中解析 `app.path().app_data_dir()`，创建目录，打开 `uclipboard.db`，执行建表；初始化 `AppState`；启动 monitor。

- [ ] **Step 2: 实现命令**

`copy_item`：从库取条目，调用 source 写回剪贴板，更新 `updated_at/copy_count`，设置 suppression，返回最新条目。`list_items` 返回带 `thumb_data_url` 的视图模型。

- [ ] **Step 3: 验证**

```bash
cd src-tauri && cargo check && cargo test
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git -c user.name="Your Name" -c user.email="you@example.com" commit -m "feat: expose clipboard commands and app state"
```

---

### Task 5: 前端数据层

**Files:** `src/lib/types.ts`, `src/lib/ipc.ts`, `src/stores/clipboard.ts`

**Interfaces:**
- `types.ts` 与 Rust `ClipItem` 字段完全一致；增加 `thumb_data_url?: string`
- `ipc.ts`：`listItems(tab, query)`, `toggleFavorite(id)`, `togglePin(id)`, `deleteItem(id)`, `clearItems()`, `copyItem(id)`, `onClipboardChanged(cb)`
- `clipboard.ts`：Zustand store，含 `items/tab/query/loading` 与动作

- [ ] **Step 1: 写类型与 IPC 封装**

```ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export const listItems = (tab: string, query: string) =>
  invoke<ClipItem[]>('list_items', { tab, query });

export const onClipboardChanged = (cb: () => void) =>
  listen('clipboard-changed', cb);
```

- [ ] **Step 2: 写 store**

```ts
type State = {
  items: ClipItem[];
  tab: TabKey;
  query: string;
  loading: boolean;
  load: () => Promise<void>;
  setTab: (tab: TabKey) => void;
  setQuery: (q: string) => void;
  toggleFavorite: (id: string) => Promise<void>;
  togglePin: (id: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
  copy: (id: string) => Promise<void>;
};
```

每个动作调用对应 IPC 后 `load()`；`setTab/setQuery` 触发 `load()`。

- [ ] **Step 3: 验证**

`npm run build` 通过 TS 检查。

- [ ] **Step 4: Commit**

```bash
git add src/lib src/stores
git -c user.name="Your Name" -c user.email="you@example.com" commit -m "feat: add frontend clipboard data layer"
```

---

### Task 6: React 界面复用 M0 视觉

**Files:** `src/App.tsx`, `src/components/*`, `src/styles.css`

**Interfaces:**
- 组件：`Titlebar`、`Tabs`、`ClipboardList`、`ClipboardItem`、`SideRail`、`PreviewModal`、`EmptyState`
- 复用 M0 的 CSS token 与布局；去掉 mock 与页面级模拟浮窗

- [ ] **Step 1: 迁移样式**

把 `prototypes/styles.css` 的主题变量、标题栏、分类、列表卡片、右侧栏、滚动条样式迁入 `src/styles.css`；删除原型专用规则。

- [ ] **Step 2: 实现组件**

- Titlebar：左上「剪贴板」、搜索、主题切换、最小化
- Tabs：全部/文本/图像/文件/收藏（仅收藏显示数量）
- ClipboardItem：文本 4 行截断/展开、图片缩略图（`thumb_data_url`）、文件列表、时间/字符数/尺寸/文件数、序号
- SideRail：复制当前、置顶当前；第三个「临时中转」按钮置灰并提示「M2 开放」
- 预览：图片大图、文本全文
- 空状态与加载态

- [ ] **Step 3: 接线**

`App` 启动时 `load()` 并订阅 `clipboard-changed`；搜索 debounce 120ms；快捷键保留 ↑↓/Tab、←→、Enter=复制、Ctrl+C、Delete、Space、Esc。

- [ ] **Step 4: 验证**

```bash
npm run build
npm run tauri dev
```

Expected：窗口打开，复制文本后列表实时出现；搜索、收藏、置顶、删除、复制生效；重启后记录仍在。

- [ ] **Step 5: Commit**

```bash
git add src index.html
git -c user.name="Your Name" -c user.email="you@example.com" commit -m "feat: build React clipboard UI"
```

---

### Task 7: M1 验收与文档

**Files:** `README.md`, `docs/superpowers/plans/2026-11-20-uclipboard-m1-acceptance.md`

- [ ] **Step 1: 全量验证**

```bash
cd src-tauri && cargo test
cd .. && npm run build
npm run tauri dev
```

手动冒烟：复制文本/图片/文件各一条；重复复制同内容只移动位置；搜索；收藏；置顶；删除；复制回剪贴板；重启应用数据仍在。

- [ ] **Step 2: 写验收记录**

记录命令结果、截图路径、已知限制（M2 功能未接）。

- [ ] **Step 3: Commit**

```bash
git add README.md docs/superpowers/plans
git -c user.name="Your Name" -c user.email="you@example.com" commit -m "docs: record M1 acceptance"
```

## Self-Review

- Spec 覆盖：M1 里程碑的三项核心（Tauri 骨架、SQLite、轮询采集、列表/搜索/收藏/置顶/删除）均有任务；快捷键/托盘/粘贴/中转窗属于 M2，未混入
- 风险：clipboard-rs 版本 API 需在安装后以 `cargo doc` 核对；若文件列表接口不可用，降级为文本+图片并在验收记录中注明
- 视觉：复用 M0 CSS，避免重新设计
