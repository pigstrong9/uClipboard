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
  copy: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>',
  logo: '<svg viewBox="0 0 24 24"><rect x="3" y="8" width="12" height="13" rx="3" fill="currentColor" opacity=".45"/><rect x="9" y="3" width="12" height="13" rx="3" fill="currentColor"/></svg>',
  close:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 6 12 12M18 6 6 18"/></svg>',
  grid: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/></svg>',
  folder:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/></svg>',
  bookmark:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 3h12a1 1 0 0 1 1 1v17l-7-4-7 4V4a1 1 0 0 1 1-1Z"/></svg>',
};

document.querySelector('.search-icon').innerHTML = ICONS.search;
document.getElementById('themeBtn').innerHTML = ICONS.sun;
document.getElementById('settingsBtn').innerHTML = ICONS.gear;
document.getElementById('moreBtn').innerHTML = ICONS.more;
document.getElementById('hideBtn').innerHTML = ICONS.close;
document.querySelector('[data-quick-action="copy"]').innerHTML = ICONS.copy;
document.querySelector('[data-quick-action="paste"]').innerHTML = ICONS.paste;
const transferBtn = document.querySelector('[data-quick-action="pin"]');
transferBtn.innerHTML = ICONS.pin;
transferBtn.title = '多选 / 临时中转';
document.getElementById('settingsTextBtn').innerHTML = ICONS.gear;
document.getElementById('clearBtn').innerHTML = ICONS.trash;

const state = {
  items: window.MOCK_ITEMS.map((item) => ({ ...item })),
  tab: 'all',
  query: '',
  selected: 0,
  expanded: new Set(),
  selectedIds: new Set(),
  transferOpen: false,
  transferIds: new Set(),
  transferQuery: '',
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
  renderTransfer();
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
  const index = Number(item.dataset.index);
  const id = item.dataset.id;
  if (event.target.closest('[data-expand]')) {
    const expandId = event.target.closest('[data-expand]').dataset.expand;
    state.expanded.has(expandId) ? state.expanded.delete(expandId) : state.expanded.add(expandId);
    return renderList();
  }
  if (state.selectedIds.has(id)) state.selectedIds.delete(id);
  else state.selectedIds.add(id);
  select(index);
});

document.getElementById('list').addEventListener('dblclick', (event) => {
  const item = event.target.closest('.item');
  if (!item) return;
  pasteItem(item.dataset.id);
});

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

document.querySelector('.quick-rail').addEventListener('click', (event) => {
  const btn = event.target.closest('[data-quick-action]');
  if (!btn) return;
  const action = btn.dataset.quickAction;
  if (action === 'pin') return openTransferPanel();
  const item = currentItem();
  if (!item) return;
  if (action === 'copy') copyItem(item.id);
  if (action === 'paste') pasteItem(item.id);
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

  const picked = state.selectedIds.has(item.id);

  return `<article class="item ${index === state.selected ? 'selected' : ''} ${picked ? 'picked' : ''}" data-id="${item.id}" data-index="${index}">
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

document.getElementById('hideBtn').addEventListener('click', hideWindow);

function pasteItem(id) {
  touchItem(id);
  renderAll();
  toast('已粘贴（原型）');
  hideWindow();
}

function copyItem(id) {
  touchItem(id);
  renderAll();
  toast('已复制');
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
  state.selectedIds.delete(id);
  state.transferIds.delete(id);
  state.selected = 0;
  renderAll();
  toast('已删除');
}

function currentItem() {
  return visibleItems()[state.selected] ?? null;
}

function select(index) {
  state.selected = index;
  renderAll();
  const el = document.querySelector('.item.selected');
  el?.scrollIntoView({ block: 'nearest' });
}

function selectedItems() {
  return state.items.filter((item) => state.selectedIds.has(item.id));
}

function transferItems() {
  return state.items.filter((item) => state.transferIds.has(item.id));
}

function openTransferPanel() {
  const picked = selectedItems();
  if (!picked.length) {
    const current = currentItem();
    if (current) state.selectedIds.add(current.id);
  }
  const toAdd = selectedItems();
  if (!toAdd.length) {
    toast('请先选择内容');
    return;
  }
  toAdd.forEach((item) => state.transferIds.add(item.id));
  state.transferOpen = true;
  state.transferQuery = '';
  renderAll();
}

function closeTransferPanel() {
  state.transferOpen = false;
  state.transferIds.clear();
  state.selectedIds.clear();
  state.transferQuery = '';
  renderAll();
}

function copyTransfer() {
  const items = transferItems();
  if (!items.length) return;
  items.forEach((item) => touchItem(item.id));
  renderAll();
  toast(`已复制 ${items.length} 条`);
}

function pasteTransfer() {
  const items = transferItems();
  if (!items.length) return;
  items.forEach((item) => touchItem(item.id));
  state.transferOpen = false;
  state.transferIds.clear();
  state.transferQuery = '';
  renderAll();
  toast(`已粘贴 ${items.length} 条（原型）`);
  hideWindow();
}

function favoriteTransfer() {
  const items = transferItems();
  if (!items.length) return;
  const allFavorite = items.every((item) => item.favorite);
  items.forEach((item) => {
    item.favorite = !allFavorite;
  });
  renderAll();
  toast(allFavorite ? '已取消收藏' : '已收藏');
}

function renderTransferItem(item) {
  const preview =
    item.kind === 'text'
      ? item.content.replace(/\s+/g, ' ').slice(0, 80)
      : item.kind === 'image'
        ? `${item.extra.width} × ${item.extra.height} 图片`
        : item.preview;
  const meta =
    item.kind === 'text'
      ? `${item.size} 字符`
      : item.kind === 'image'
        ? '图片'
        : `${item.extra.count} 个文件`;
  const icon = item.kind === 'text' ? ICONS.text : item.kind === 'image' ? ICONS.image : ICONS.files;

  return `<div class="transfer-item">
    <span class="transfer-kind">${icon}</span>
    <div class="transfer-text">
      <p>${escapeHtml(preview)}</p>
      <span>${formatRelativeTime(item.updated_at)} · ${meta}</span>
    </div>
    <button class="transfer-remove" data-transfer-remove="${item.id}" title="移出">×</button>
  </div>`;
}

function renderTransfer() {
  const panel = document.getElementById('transferPanel');
  if (!state.transferOpen) {
    panel.hidden = true;
    panel.innerHTML = '';
    transferBtn.dataset.count = String(state.selectedIds.size);
    transferBtn.classList.remove('active');
    return;
  }

  const total = state.transferIds.size;
  const keyword = state.transferQuery.trim().toLowerCase();
  const items = transferItems().filter((item) => {
    if (!keyword) return true;
    const text = item.kind === 'files' ? item.content.join('\n') : item.content;
    return text.toLowerCase().includes(keyword);
  });

  panel.hidden = false;
  panel.innerHTML = `
    <div class="transfer-head">
      <span class="transfer-pin">${ICONS.pin}</span>
      <span class="transfer-title">临时中转${total ? ` · ${total}` : ''}</span>
      <button class="icon-btn" data-transfer-action="copy" title="复制全部" ${total ? '' : 'disabled'}>${ICONS.copy}</button>
      <button class="icon-btn" data-transfer-action="favorite" title="收藏全部" ${total ? '' : 'disabled'}>${ICONS.star}</button>
      <button class="icon-btn" data-transfer-action="search" title="搜索">${ICONS.search}</button>
      <button class="icon-btn" data-transfer-action="close" title="关闭">${ICONS.close}</button>
    </div>
    <div class="transfer-search">
      <input id="transferSearch" type="text" placeholder="在中转内容中搜索…" value="${escapeHtml(state.transferQuery)}" />
    </div>
    <div class="transfer-body">
      ${
        items.length
          ? items.map(renderTransferItem).join('')
          : `<div class="transfer-empty">${total ? '没有匹配的内容' : '先在列表中选择条目，再点击中转按钮'}</div>`
      }
    </div>
    <div class="transfer-foot">
      <button class="transfer-btn" data-transfer-action="clear" ${total ? '' : 'disabled'}>清空</button>
      <button class="transfer-btn" data-transfer-action="copy" ${total ? '' : 'disabled'}>复制全部</button>
      <button class="transfer-btn primary" data-transfer-action="paste" ${total ? '' : 'disabled'}>粘贴全部</button>
    </div>`;

  transferBtn.dataset.count = String(total);
  transferBtn.classList.add('active');
}

const transferPanelEl = document.getElementById('transferPanel');
transferPanelEl.addEventListener('click', (event) => {
  const remove = event.target.closest('[data-transfer-remove]');
  if (remove) {
    state.transferIds.delete(remove.dataset.transferRemove);
    renderAll();
    return;
  }
  const action = event.target.closest('[data-transfer-action]');
  if (!action) return;
  const name = action.dataset.transferAction;
  if (name === 'copy') return copyTransfer();
  if (name === 'paste') return pasteTransfer();
  if (name === 'favorite') return favoriteTransfer();
  if (name === 'close') return closeTransferPanel();
  if (name === 'clear') {
    state.transferIds.clear();
    renderAll();
    return;
  }
  if (name === 'search') document.getElementById('transferSearch')?.focus();
});

transferPanelEl.addEventListener('input', (event) => {
  if (event.target.id !== 'transferSearch') return;
  state.transferQuery = event.target.value;
  renderTransfer();
  const input = document.getElementById('transferSearch');
  if (input) {
    input.focus();
    input.setSelectionRange(input.value.length, input.value.length);
  }
});

document.addEventListener('keydown', (event) => {
  const target = event.target;
  const inSearch = target instanceof HTMLElement && target.id === 'searchInput';
  const meta = event.metaKey || event.ctrlKey;

  if (event.key === 'Escape') {
    if (!document.getElementById('settings').hidden) return closeSettings();
    if (!document.getElementById('modal').hidden) return closeModal();
    if (!document.getElementById('contextMenu').hidden) return closeContextMenu();
    if (state.transferOpen) return closeTransferPanel();
    if (state.selectedIds.size) {
      state.selectedIds.clear();
      renderAll();
      return;
    }
    if (state.query) {
      state.query = '';
      document.getElementById('searchInput').value = '';
      renderList();
      return;
    }
    if (inSearch) return target.blur();
    return hideWindow();
  }

  if (!document.getElementById('settings').hidden) return;
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

const params = new URLSearchParams(window.location.search);
const themeParam = params.get('theme');
if (themeParam === 'light' || themeParam === 'dark') setTheme(themeParam);
const tabParam = params.get('tab');
if (TABS.some((tab) => tab.key === tabParam)) state.tab = tabParam;
if (params.get('settings') === '1') openSettings();
if (params.get('transfer') === '1') {
  state.items.slice(0, 2).forEach((item) => {
    state.selectedIds.add(item.id);
    state.transferIds.add(item.id);
  });
  state.transferOpen = true;
}

renderAll();
