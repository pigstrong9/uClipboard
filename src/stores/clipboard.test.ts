import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/ipc', () => ({
  listItems: vi.fn(),
  favoriteCount: vi.fn(),
  stats: vi.fn(),
  openTransfer: vi.fn(),
  toggleFavorite: vi.fn(),
  togglePin: vi.fn(),
  deleteItem: vi.fn(),
  copyItem: vi.fn(),
  pasteItem: vi.fn(),
  clearItems: vi.fn(),
}));

import * as ipc from '../lib/ipc';
import type { ClipSummary } from '../lib/types';

import { useClipboard } from './clipboard';

function summary(id: string): ClipSummary {
  return {
    id,
    kind: 'text',
    preview: id,
    text: id,
    truncated: false,
    files: [],
    width: null,
    height: null,
    file_count: null,
    size: 1,
    source_app: null,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: 1,
    updated_at: 1,
  };
}

describe('clipboard store', () => {
  beforeEach(() => {
    vi.mocked(ipc.listItems).mockResolvedValue([summary('text-item')]);
    vi.mocked(ipc.favoriteCount).mockResolvedValue(0);
    vi.mocked(ipc.stats).mockResolvedValue(1);
    useClipboard.setState({
      items: [summary('a'), summary('b')],
      tab: 'all',
      query: '',
      loading: false,
      error: null,
      selectedId: 'a',
      selectedIds: ['a', 'b'],
      favoriteCount: 0,
      totalCount: 2,
    });
  });

  it('drops multi-selection when switching tabs', async () => {
    useClipboard.getState().setTab('text');

    expect(useClipboard.getState().selectedIds).toEqual([]);
    await vi.waitFor(() => expect(ipc.listItems).toHaveBeenCalledWith('text', ''));
  });

  it('does not auto-select the first row when nothing is selected', async () => {
    useClipboard.setState({ selectedId: null, items: [] });

    await useClipboard.getState().load();

    expect(useClipboard.getState().selectedId).toBeNull();
  });

  it('resetView returns to the all tab and issues a single reload', async () => {
    useClipboard.setState({
      tab: 'favorite',
      query: 'abc',
      selectedId: 'a',
      selectedIds: ['a', 'b'],
    });

    useClipboard.getState().resetView();

    const state = useClipboard.getState();
    expect(state.tab).toBe('all');
    expect(state.query).toBe('');
    expect(state.selectedId).toBeNull();
    expect(state.selectedIds).toEqual([]);
    await vi.waitFor(() => expect(ipc.listItems).toHaveBeenCalledWith('all', ''));
    expect(ipc.listItems).toHaveBeenCalledTimes(1);
  });
});
