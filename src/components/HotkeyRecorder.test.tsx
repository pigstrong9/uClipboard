// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/useI18n', async () => {
  const { createTranslate } = await import('../lib/i18n');
  return {
    useI18n: () => ({ language: 'en-US', t: createTranslate('en-US') }),
  };
});

vi.mock('../lib/ipc', () => ({
  beginHotkeyCapture: vi.fn(),
  endHotkeyCapture: vi.fn(),
}));

import * as ipc from '../lib/ipc';

import { HotkeyRecorder } from './HotkeyRecorder';

describe('HotkeyRecorder', () => {
  beforeEach(() => {
    vi.mocked(ipc.beginHotkeyCapture).mockResolvedValue(undefined);
    vi.mocked(ipc.endHotkeyCapture).mockResolvedValue(undefined);
  });

  afterEach(cleanup);

  it('records a pressed combination and commits it', async () => {
    const onCommit = vi.fn();
    render(<HotkeyRecorder value="Alt+V" onCommit={onCommit} />);

    const button = screen.getByRole('button', { name: 'Alt+V' });
    fireEvent.click(button);
    fireEvent.keyDown(button, {
      key: 'A',
      code: 'KeyA',
      ctrlKey: true,
      shiftKey: true,
    });

    await waitFor(() => expect(onCommit).toHaveBeenCalledWith('Ctrl+Shift+A'));
    expect(ipc.endHotkeyCapture).toHaveBeenCalled();
  });

  it('records keys fired outside the button, where macOS WKWebView sends them', async () => {
    const onCommit = vi.fn();
    render(<HotkeyRecorder value="Alt+V" onCommit={onCommit} />);

    fireEvent.click(screen.getByRole('button', { name: 'Alt+V' }));
    fireEvent.keyDown(document.body, {
      key: 'å',
      code: 'KeyA',
      altKey: true,
    });

    await waitFor(() => expect(onCommit).toHaveBeenCalledWith('Alt+A'));
  });

  it('cancels with Escape without committing', async () => {
    const onCommit = vi.fn();
    render(<HotkeyRecorder value="Alt+V" onCommit={onCommit} />);

    const button = screen.getByRole('button', { name: 'Alt+V' });
    fireEvent.click(button);
    fireEvent.keyDown(button, { key: 'Escape', code: 'Escape' });

    await waitFor(() => expect(ipc.endHotkeyCapture).toHaveBeenCalled());
    expect(onCommit).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Alt+V' })).toBeTruthy();
  });
});
