// @vitest-environment jsdom
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `asset://${path}`,
}));

vi.mock('../lib/ipc', () => ({
  getItem: vi.fn(),
}));

vi.mock('../lib/useI18n', async () => {
  const { createTranslate } = await import('../lib/i18n');
  return {
    useI18n: () => ({ language: 'en-US', t: createTranslate('en-US') }),
  };
});

import * as ipc from '../lib/ipc';
import type { ClipItem, ClipSummary } from '../lib/types';

import { ClipboardList } from './ClipboardList';

function summary(overrides: Partial<ClipSummary>): ClipSummary {
  return {
    id: 'a',
    kind: 'text',
    preview: 'preview',
    text: 'preview',
    truncated: false,
    files: [],
    width: null,
    height: null,
    file_count: null,
    size: 8,
    source_app: null,
    favorite: false,
    pinned: false,
    copy_count: 1,
    created_at: 1,
    updated_at: Date.now(),
    ...overrides,
  };
}

const noop = () => undefined;

const baseProps = {
  selectedId: null,
  selectedIds: [],
  loading: false,
  onSelect: noop,
  onCopy: noop,
  onPaste: noop,
  onFavorite: noop,
  onPin: noop,
  onDelete: noop,
};

describe('ClipboardList', () => {
  beforeEach(() => {
    vi.mocked(ipc.getItem).mockReset();
    Element.prototype.scrollIntoView = vi.fn();
  });

  afterEach(cleanup);

  it('exposes listbox semantics and asset thumbnails', () => {
    const item = summary({
      kind: 'image',
      thumb_path: '/app/images/a_thumb.jpg',
      width: 120,
      height: 80,
    });

    render(
      <ClipboardList
        {...baseProps}
        items={[item]}
        selectedId="a"
        selectedIds={['a']}
      />,
    );

    expect(screen.getByRole('listbox')).toBeTruthy();
    expect(screen.getByRole('option', { selected: true })).toBeTruthy();
    const image = screen.getByAltText('Clipboard image') as HTMLImageElement;
    expect(image.getAttribute('src')).toBe('asset:///app/images/a_thumb.jpg');
    expect(screen.getByText('120 × 80')).toBeTruthy();
  });

  it('fetches the full text only when a truncated row is expanded', async () => {
    const user = userEvent.setup();
    vi.mocked(ipc.getItem).mockResolvedValue({
      ...summary({ id: 't1', size: 1000, truncated: true }),
      content: 'FULL CONTENT',
      extra: null,
    } as ClipItem);

    render(
      <ClipboardList
        {...baseProps}
        items={[summary({ id: 't1', text: 'short text', truncated: true, size: 1000 })]}
      />,
    );

    expect(screen.queryByText('FULL CONTENT')).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Expand' }));

    await waitFor(() => expect(screen.getByText('FULL CONTENT')).toBeTruthy());
    expect(ipc.getItem).toHaveBeenCalledWith('t1');
  });
});
