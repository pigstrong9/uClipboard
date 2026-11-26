import { useEffect, useState, type FocusEvent } from 'react';
import { getVersion } from '@tauri-apps/api/app';

import logoUrl from '../assets/logo.png';
import { errorText } from '../lib/errors';
import { isMacPlatform } from '../lib/hotkey';
import * as ipc from '../lib/ipc';
import { HotkeyRecorder } from './HotkeyRecorder';
import { Icon } from '../lib/icons';
import { useI18n } from '../lib/useI18n';
import type { Settings, UpdateInfo } from '../lib/types';
import { useClipboard } from '../stores/clipboard';
import { useSettings } from '../stores/settings';
import { useUi } from '../stores/ui';

const SUGGESTED_APPS_WINDOWS = [
  'KeePass.exe',
  'KeePassXC.exe',
  '1Password.exe',
  'Bitwarden.exe',
  'LastPass.exe',
  'Dashlane.exe',
  'Keeper.exe',
  'Enpass.exe',
  'NordPass.exe',
];

const SUGGESTED_APPS_MACOS = [
  'KeePassXC',
  '1Password',
  'Bitwarden',
  'Dashlane',
  'Keeper',
  'Enpass',
  'NordPass',
];

type Section = 'basic' | 'history' | 'privacy' | 'about';

interface Props {
  onClose: () => void;
}

export function SettingsView({ onClose }: Props) {
  const { t } = useI18n();
  const settings = useSettings((state) => state.settings);
  const save = useSettings((state) => state.save);
  const toast = useUi((state) => state.toast);
  const ask = useUi((state) => state.ask);
  const [section, setSection] = useState<Section>('basic');
  const [ignored, setIgnored] = useState('');
  const [version, setVersion] = useState('');
  const [checking, setChecking] = useState(false);
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [accessibility, setAccessibility] = useState<boolean | null>(null);
  const [screenPermission, setScreenPermission] = useState<boolean | null>(null);

  const persistedIgnored = settings?.ignoredApps.join('\n');

  useEffect(() => {
    if (persistedIgnored === undefined) return;
    setIgnored(persistedIgnored);
  }, [persistedIgnored]);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    ipc
      .accessibilityStatus()
      .then(setAccessibility)
      .catch(() => setAccessibility(null));
  }, []);

  useEffect(() => {
    ipc
      .screenshotPermissionStatus()
      .then(setScreenPermission)
      .catch(() => setScreenPermission(null));
  }, []);

  const commit = async (patch: Partial<Settings>, message?: string) => {
    try {
      await save(patch);
      if (message) toast(message);
    } catch (error) {
      toast(errorText(t, error));
    }
  };

  const commitNumber = (
    event: FocusEvent<HTMLInputElement>,
    current: number,
    apply: (value: number) => Partial<Settings>,
  ) => {
    const value = Number(event.target.value);
    if (!Number.isFinite(value)) {
      event.target.value = String(current);
      return;
    }
    void commit(apply(value));
  };

  if (!settings) {
    return (
      <section className="settings" role="dialog" aria-label={t('settingsTitle')}>
        <div className="settings-body">
          <p>{t('loading')}</p>
        </div>
      </section>
    );
  }

  const applyIgnored = () => {
    void commit(
      {
        ignoredApps: ignored
          .split(/\r?\n/)
          .map((value) => value.trim())
          .filter(Boolean),
      },
      t('ignoredSaved'),
    );
  };

  const addSuggestion = (app: string) => {
    const current = ignored
      .split(/\r?\n/)
      .map((value) => value.trim().toLowerCase())
      .filter(Boolean);
    if (current.includes(app.toLowerCase())) return;
    setIgnored((value) => (value.trim() ? `${value.trimEnd()}\n${app}` : app));
  };

  const checkUpdate = async () => {
    setChecking(true);
    setUpdate(null);
    try {
      const found = await ipc.checkUpdate();
      if (found) {
        setUpdate(found);
        toast(t('updateFound', { version: found.version }));
      } else {
        toast(t('upToDate'));
      }
    } catch (error) {
      toast(t('updateFailed', { error: errorText(t, error) }));
    } finally {
      setChecking(false);
    }
  };

  const openReleasePage = async () => {
    try {
      await ipc.openReleasePage();
    } catch (error) {
      toast(t('openReleaseFailed', { error: errorText(t, error) }));
    }
  };

  const askClear = () => {
    ask({
      message: t('confirmClearAllData'),
      onConfirm: () => {
        void useClipboard
          .getState()
          .clear()
          .then(() => toast(t('allDataCleared')))
          .catch((cause) => toast(errorText(t, cause)));
      },
    });
  };

  const title =
    section === 'basic'
      ? t('navBasic')
      : section === 'history'
        ? t('navHistory')
        : section === 'privacy'
          ? t('navPrivacy')
          : t('navAbout');

  return (
    <section className="settings" role="dialog" aria-label={t('settingsTitle')}>
      <nav className="settings-nav">
        {(
          [
            ['basic', 'navBasic'],
            ['history', 'navHistory'],
            ['privacy', 'navPrivacy'],
            ['about', 'navAbout'],
          ] as [Section, string][]
        ).map(([key, label]) => (
          <button
            key={key}
            type="button"
            className={section === key ? 'active' : ''}
            onClick={() => setSection(key)}
          >
            {t(label)}
          </button>
        ))}
        <button type="button" className="settings-back" onClick={onClose}>
          <Icon name="close" />
          <span>{t('back')}</span>
        </button>
      </nav>

      <div className="settings-body">
        <div className="settings-heading">
          <h2>{title}</h2>
          <button type="button" className="icon-btn" title={t('close')} onClick={onClose}>
            <Icon name="close" />
          </button>
        </div>

        {section === 'basic' && (
          <div className="settings-group">
            <div className="setting-row setting-stack">
              <div>
                <strong>{t('basicHotkey')}</strong>
                <span className="settings-note">{t('basicHotkeyNote')}</span>
              </div>
              <HotkeyRecorder
                value={settings.hotkey}
                onCommit={(hotkey) => void commit({ hotkey }, t('hotkeySaved'))}
              />
            </div>

            <div className="setting-row setting-stack">
              <div>
                <strong>{t('screenshotHotkey')}</strong>
                <span className="settings-note">{t('screenshotHotkeyNote')}</span>
              </div>
              <HotkeyRecorder
                value={settings.screenshotHotkey}
                onCommit={(screenshotHotkey) =>
                  void commit({ screenshotHotkey }, t('hotkeySaved'))
                }
              />
            </div>

            {accessibility === false && (
              <div className="setting-row setting-stack">
                <span className="settings-note">{t('accessibilityNote')}</span>
                <div className="settings-inline">
                  <button
                    type="button"
                    className="settings-button"
                    onClick={() => {
                      void ipc
                        .requestAccessibilityPermission()
                        .then(async (trusted) => {
                          setAccessibility(trusted);
                          if (!trusted) {
                            await ipc.openAccessibilitySettings().catch(() => undefined);
                          }
                          toast(
                            t(trusted ? 'accessibilityGranted' : 'accessibilityRequested'),
                          );
                        })
                        .catch(() => undefined);
                    }}
                  >
                    {t('requestAccessibility')}
                  </button>
                  <button
                    type="button"
                    className="settings-button"
                    onClick={() => void ipc.openAccessibilitySettings()}
                  >
                    {t('openAccessibilitySettings')}
                  </button>
                </div>
              </div>
            )}

            {screenPermission === false && (
              <div className="setting-row setting-stack">
                <span className="settings-note">{t('screenRecordingNote')}</span>
                <div className="settings-inline">
                  <button
                    type="button"
                    className="settings-button"
                    onClick={() => {
                      void ipc
                        .requestScreenshotPermission()
                        .then(async (granted) => {
                          setScreenPermission(granted);
                          if (!granted) {
                            await ipc.openScreenRecordingSettings().catch(() => undefined);
                          }
                          toast(
                            t(granted ? 'screenRecordingGranted' : 'screenRecordingNote'),
                          );
                        })
                        .catch(() => undefined);
                    }}
                  >
                    {t('requestScreenRecording')}
                  </button>
                  <button
                    type="button"
                    className="settings-button"
                    onClick={() => void ipc.openScreenRecordingSettings()}
                  >
                    {t('openScreenRecordingSettings')}
                  </button>
                </div>
              </div>
            )}

            <label className="setting-row">
              <span>{t('autoStart')}</span>
              <input
                type="checkbox"
                checked={settings.autoStart}
                onChange={(event) => void commit({ autoStart: event.target.checked })}
              />
            </label>
            <span className="settings-note settings-note-row">{t('autoStartNote')}</span>

            <label className="setting-row">
              <span>{t('hideOnBlur')}</span>
              <input
                type="checkbox"
                checked={settings.hideOnBlur}
                onChange={(event) => void commit({ hideOnBlur: event.target.checked })}
              />
            </label>
            <label className="setting-row">
              <span>{t('pastePlainText')}</span>
              <input
                type="checkbox"
                checked={settings.pastePlainText}
                onChange={(event) => void commit({ pastePlainText: event.target.checked })}
              />
            </label>

            <label className="setting-row">
              <span>{t('theme')}</span>
              <select
                className="settings-select"
                value={settings.theme}
                onChange={(event) =>
                  void commit({ theme: event.target.value as Settings['theme'] })
                }
              >
                <option value="system">{t('themeSystem')}</option>
                <option value="light">{t('themeLight')}</option>
                <option value="dark">{t('themeDark')}</option>
              </select>
            </label>
            <label className="setting-row">
              <span>{t('language')}</span>
              <select
                className="settings-select"
                value={settings.language}
                onChange={(event) =>
                  void commit({ language: event.target.value as Settings['language'] })
                }
              >
                <option value="system">{t('languageSystem')}</option>
                <option value="zh-CN">{t('langZh')}</option>
                <option value="en-US">{t('langEn')}</option>
              </select>
            </label>
          </div>
        )}

        {section === 'history' && (
          <div className="settings-group">
            <label className="setting-row">
              <span>{t('historyLimit')}</span>
              <input
                type="number"
                className="settings-number"
                min={0}
                max={10000}
                defaultValue={settings.historyLimit}
                onBlur={(event) =>
                  commitNumber(event, settings.historyLimit, (value) => ({ historyLimit: value }))
                }
              />
            </label>
            <span className="settings-note settings-note-row">{t('historyLimitNote')}</span>

            <label className="setting-row">
              <span>{t('retentionDays')}</span>
              <input
                type="number"
                className="settings-number"
                min={0}
                max={3650}
                defaultValue={settings.retentionDays}
                onBlur={(event) =>
                  commitNumber(event, settings.retentionDays, (value) => ({ retentionDays: value }))
                }
              />
            </label>
            <span className="settings-note settings-note-row">{t('retentionDaysNote')}</span>

            <label className="setting-row">
              <span>{t('recordText')}</span>
              <input
                type="checkbox"
                checked={settings.recordText}
                onChange={(event) => void commit({ recordText: event.target.checked })}
              />
            </label>
            <label className="setting-row">
              <span>{t('recordImages')}</span>
              <input
                type="checkbox"
                checked={settings.recordImages}
                onChange={(event) => void commit({ recordImages: event.target.checked })}
              />
            </label>
            <label className="setting-row">
              <span>{t('recordFiles')}</span>
              <input
                type="checkbox"
                checked={settings.recordFiles}
                onChange={(event) => void commit({ recordFiles: event.target.checked })}
              />
            </label>
            <label className="setting-row">
              <span>{t('minTextLength')}</span>
              <input
                type="number"
                className="settings-number"
                min={1}
                max={100}
                defaultValue={settings.minTextLength}
                onBlur={(event) =>
                  commitNumber(event, settings.minTextLength, (value) => ({ minTextLength: value }))
                }
              />
            </label>
            <label className="setting-row">
              <span>{t('trimText')}</span>
              <input
                type="checkbox"
                checked={settings.trimText}
                onChange={(event) => void commit({ trimText: event.target.checked })}
              />
            </label>
            <span className="settings-note settings-note-row">{t('trimTextNote')}</span>

            <label className="setting-row">
              <span>{t('monitorPaused')}</span>
              <input
                type="checkbox"
                checked={settings.monitorPaused}
                onChange={(event) => void commit({ monitorPaused: event.target.checked })}
              />
            </label>
          </div>
        )}

        {section === 'privacy' && (
          <div className="settings-group">
            <div className="setting-row setting-stack">
              <div>
                <strong>{t('ignoredApps')}</strong>
                <span className="settings-note">{t('ignoredAppsNote')}</span>
              </div>
              <textarea
                className="settings-textarea"
                rows={8}
                value={ignored}
                onChange={(event) => setIgnored(event.target.value)}
              />
              <div className="settings-inline settings-inline-wrap">
                <span className="settings-note">{t('ignoredSuggestions')}</span>
                {(isMacPlatform() ? SUGGESTED_APPS_MACOS : SUGGESTED_APPS_WINDOWS).map((app) => (
                  <button
                    key={app}
                    type="button"
                    className="settings-chip"
                    onClick={() => addSuggestion(app)}
                  >
                    {app}
                  </button>
                ))}
              </div>
              <div className="settings-inline">
                <button type="button" className="settings-button" onClick={applyIgnored}>
                  {t('applyIgnored')}
                </button>
              </div>
            </div>
            <p className="settings-note settings-note-row">{t('privacyNote')}</p>
            <div className="settings-inline">
              <button
                type="button"
                className="settings-button danger"
                onClick={askClear}
              >
                {t('clearAllData')}
              </button>
            </div>
          </div>
        )}

        {section === 'about' && (
          <div className="settings-group">
            <div className="about-card">
              <div className="about-brand">
                <img className="about-logo" src={logoUrl} alt="" />
                <strong>uClipboard</strong>
              </div>
              <p>{t('aboutDescription')}</p>
              <div className="about-meta">
                <span>
                  {t('aboutVersion')}: {version || '—'}
                </span>
                <span>{t('aboutLicense')}: Apache-2.0</span>
              </div>
            </div>
            <label className="setting-row">
              <span>{t('autoUpdate')}</span>
              <input
                type="checkbox"
                checked={settings.autoUpdate}
                onChange={(event) => void commit({ autoUpdate: event.target.checked })}
              />
            </label>
            <div className="settings-inline">
              <button
                type="button"
                className="settings-button"
                onClick={() => void checkUpdate()}
                disabled={checking}
              >
                {checking ? t('checking') : t('checkUpdate')}
              </button>
              {update && (
                <button
                  type="button"
                  className="settings-button"
                  onClick={() => void openReleasePage()}
                >
                  {t('openReleasePage')}
                </button>
              )}
            </div>
            {update && (
              <p className="settings-note settings-note-row">
                {t('updateFound', { version: update.version })}
              </p>
            )}
            <div className="settings-inline">
              <button
                type="button"
                className="settings-button"
                onClick={() => {
                  void ipc.openLogsDir().catch((error) => toast(errorText(t, error)));
                }}
              >
                {t('openLogsDir')}
              </button>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
