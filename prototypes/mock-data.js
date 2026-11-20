const now = Date.now();
const min = 60 * 1000;
const day = 24 * 60 * min;

function svgImage(width, height, label) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="480" height="300" viewBox="0 0 480 300">
    <rect width="480" height="300" rx="10" fill="#0d1117"/>
    <rect x="1" y="1" width="478" height="28" rx="9" fill="#161b22"/>
    <circle cx="18" cy="15" r="4" fill="#ff5f57"/>
    <circle cx="32" cy="15" r="4" fill="#febc2e"/>
    <circle cx="46" cy="15" r="4" fill="#28c840"/>
    <text x="64" y="19" font-family="Consolas, monospace" font-size="11" fill="#8b949e">${label}</text>
    <rect x="16" y="46" width="240" height="10" rx="3" fill="#30363d"/>
    <rect x="16" y="66" width="330" height="10" rx="3" fill="#30363d"/>
    <rect x="16" y="86" width="160" height="10" rx="3" fill="#7dd3fc"/>
    <rect x="16" y="106" width="380" height="10" rx="3" fill="#30363d"/>
    <rect x="16" y="126" width="280" height="10" rx="3" fill="#f0b429"/>
    <rect x="16" y="146" width="350" height="10" rx="3" fill="#30363d"/>
    <rect x="16" y="166" width="200" height="10" rx="3" fill="#7ee787"/>
    <rect x="16" y="186" width="320" height="10" rx="3" fill="#30363d"/>
  </svg>`;
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

window.MOCK_ITEMS = [
  {
    id: 'itm-001',
    kind: 'image',
    content: 'mock://image/screenshot',
    extra: { width: 1205, height: 903, thumb: svgImage(1205, 903, 'screenshot.png') },
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
      'C:/Users/example/Documents/项目说明.pdf',
      'C:/Users/example/Downloads/设计规范-v2.pdf',
      'C:/Users/example/Downloads/demo-assets.zip',
      'C:/Users/example/Downloads/release-notes.md',
    ],
    extra: { count: 4 },
    preview: '项目说明.pdf 等 4 个文件',
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
    content: `export async function readClipboard(): Promise<ClipItem | null> {
  const text = await navigator.clipboard.readText()
  if (!text.trim()) return null
  return { id: crypto.randomUUID(), kind: 'text', content: text }
}`,
    preview: 'export async function readClipboard…',
    size: 210,
    favorite: false,
    pinned: false,
    copy_count: 4,
    created_at: now - 9 * min,
    updated_at: now - 9 * min,
  },
  {
    id: 'itm-004',
    kind: 'text',
    content: '{ "historyLimit": 500, "retentionDays": 30, "pastePlainText": false }',
    preview: '{ "historyLimit": 500, "retentionDays": 30, …',
    size: 72,
    favorite: true,
    pinned: false,
    copy_count: 1,
    created_at: now - 10 * min,
    updated_at: now - 10 * min,
  },
  {
    id: 'itm-005',
    kind: 'text',
    content: 'pnpm install\npnpm tauri dev\npnpm tauri build',
    preview: 'pnpm install / pnpm tauri dev / pnpm tauri build',
    size: 44,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 23 * min,
    updated_at: now - 23 * min,
  },
  {
    id: 'itm-006',
    kind: 'text',
    content: 'https://example.com/docs/clipboard-api',
    preview: 'https://example.com/docs/clipboard-api',
    size: 39,
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
      '# 剪贴板工具开发笔记\n\n1. 监听系统剪贴板变化\n2. 内容去重与置顶\n3. 全局快捷键唤起\n\n记录一次周末原型开发的思路。',
    preview: '# 剪贴板工具开发笔记',
    size: 72,
    favorite: true,
    pinned: false,
    copy_count: 2,
    created_at: now - 2 * day,
    updated_at: now - 2 * day,
  },
  {
    id: 'itm-008',
    kind: 'text',
    content: '这是一段用于测试长文本展开与收起的示例内容。'.repeat(20),
    preview: '这是一段用于测试长文本展开与收起的示例内容。'.repeat(3),
    size: 440,
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
    extra: { width: 1920, height: 1080, thumb: svgImage(1920, 1080, 'usage-chart.png') },
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
    content: ['D:/workspace/demo/README.md', 'D:/workspace/demo/package.json'],
    extra: { count: 2 },
    preview: 'README.md 等 2 个文件',
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
    content: 'SELECT kind, COUNT(*) AS total FROM items GROUP BY kind ORDER BY total DESC;',
    preview: 'SELECT kind, COUNT(*) AS total FROM items GROUP BY kind …',
    size: 74,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 42 * day,
    updated_at: now - 42 * day,
  },
  {
    id: 'itm-012',
    kind: 'text',
    content: 'git commit -m "feat: clipboard history"',
    preview: 'git commit -m "feat: clipboard history"',
    size: 41,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: now - 50 * day,
    updated_at: now - 50 * day,
  },
];
