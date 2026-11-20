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
    content:
      'window.localStorage.setItem("records-removed", JSON.stringify(recordItems.map(x => x.hash)))',
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
    content:
      'https://github.com/pigstrong9/LoveqPodcast/releases/download/v1.4/app-release.apk',
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
    content:
      '#为了上下班路上听播客，我花了 2 天时间开发了一个 APP\n\n《一些事一些情》这个电台节目，我从高中就开始听了，断断续续十几年，到现在还是每天上下班开车路上的固定搭子。\n\n但说实话，现在想好好听个节目，还真没那么简单。',
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
    content: [
      'D:/workspace/uClipboard/prototypes/index.html',
      'D:/workspace/uClipboard/prototypes/app.js',
    ],
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
