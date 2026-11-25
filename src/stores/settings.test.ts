import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/ipc', () => ({
  getSettings: vi.fn(),
  updateSettings: vi.fn(),
  onSettingsChanged: vi.fn(),
  toggleMonitoring: vi.fn(),
}));

import * as ipc from '../lib/ipc';
import type { Settings } from '../lib/types';

import { useSettings } from './settings';

function settings(ignoredApps: string[] = []): Settings {
  return {
    hotkey: 'Alt+V',
    screenshotHotkey: 'Alt+A',
    autoStart: false,
    hideOnBlur: true,
    theme: 'system',
    language: 'system',
    historyLimit: 500,
    retentionDays: 30,
    recordText: true,
    recordImages: true,
    recordFiles: true,
    minTextLength: 2,
    ignoredApps,
    pastePlainText: false,
    autoUpdate: true,
    monitorPaused: false,
    maxTextBytes: 5 * 1024 * 1024,
    maxImageBytes: 20 * 1024 * 1024,
  };
}

describe('settings store', () => {
  beforeEach(() => {
    vi.mocked(ipc.updateSettings).mockReset();
    vi.mocked(ipc.updateSettings).mockImplementation(async (value) => value);
    useSettings.setState({ settings: settings(), loading: false, error: null });
  });

  it('appends a new source app to the ignored list', async () => {
    const added = await useSettings.getState().ignoreApp(' GameViewer.exe ');

    expect(added).toBe(true);
    expect(ipc.updateSettings).toHaveBeenCalledWith(
      expect.objectContaining({ ignoredApps: ['GameViewer.exe'] }),
    );
    expect(useSettings.getState().settings?.ignoredApps).toEqual(['GameViewer.exe']);
  });

  it('does not duplicate an app that differs only by case', async () => {
    useSettings.setState({ settings: settings(['gameviewer.exe', 'KeePass.exe']) });

    const added = await useSettings.getState().ignoreApp('GameViewer.exe');

    expect(added).toBe(false);
    expect(ipc.updateSettings).not.toHaveBeenCalled();
    expect(useSettings.getState().settings?.ignoredApps).toEqual([
      'gameviewer.exe',
      'KeePass.exe',
    ]);
  });

  it('serializes two rapid additions without losing either', async () => {
    const first = useSettings.getState().ignoreApp('GameViewer.exe');
    const second = useSettings.getState().ignoreApp('WorkBuddy.exe');
    await Promise.all([first, second]);

    expect(useSettings.getState().settings?.ignoredApps).toEqual([
      'GameViewer.exe',
      'WorkBuddy.exe',
    ]);
  });
});
