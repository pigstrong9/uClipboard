const ICONS = {
  clipboard:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>',
  layers:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m12 2 9 5-9 5-9-5 9-5Z"/><path d="m3 12 9 5 9-5"/><path d="m3 17 9 5 9-5"/></svg>',
  text: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>',
  image:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>',
  files:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/></svg>',
  star: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m12 2 3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2Z"/></svg>',
  file: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/></svg>',
  pin: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16h14v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1Z"/></svg>',
  paste:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1Z"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>',
  trash:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M10 11v6"/><path d="M14 11v6"/></svg>',
  gear: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"/></svg>',
  more: '<svg viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.6"/><circle cx="12" cy="12" r="1.6"/><circle cx="19" cy="12" r="1.6"/></svg>',
  sun: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>',
  search:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/></svg>',
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

document.getElementById('list').addEventListener('click', (event) => {
  const btn = event.target.closest('[data-expand]');
  if (!btn) return;
  const id = btn.dataset.expand;
  state.expanded.has(id) ? state.expanded.delete(id) : state.expanded.add(id);
  renderList();
});

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

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

renderAll();
