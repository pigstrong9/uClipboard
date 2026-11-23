import { convertFileSrc } from '@tauri-apps/api/core';
import { useCallback, useEffect, useRef, useState, type PointerEvent } from 'react';

import { errorText } from './lib/errors';
import * as ipc from './lib/ipc';
import { selectionToPixels, type SelectionBox } from './lib/screenshot';
import type { ScreenshotSession } from './lib/types';
import { useI18n } from './lib/useI18n';
import { useSettings } from './stores/settings';

export default function ScreenshotApp() {
  const [session, setSession] = useState<ScreenshotSession | null>(null);
  const [ready, setReady] = useState(false);
  const [selection, setSelection] = useState<SelectionBox | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const dragStart = useRef<{ x: number; y: number } | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const initSettings = useSettings((state) => state.init);
  const { t } = useI18n();

  useEffect(() => {
    void initSettings();
  }, [initSettings]);

  /**
   * The overlay's own box in CSS pixels. Measured from the element rather than from
   * `window.innerWidth` because the window is resized by the backend after the page
   * loads, and a stale size would map the selection through the wrong scale.
   */
  const viewSize = () => {
    const rect = rootRef.current?.getBoundingClientRect();
    if (!rect || rect.width < 1 || rect.height < 1) return null;
    return { width: rect.width, height: rect.height };
  };

  useEffect(() => {
    let cancelled = false;
    ipc
      .screenshotSession()
      .then((value) => {
        if (!cancelled) setSession(value);
      })
      .catch((cause) => {
        if (!cancelled) {
          setError(errorText(t, cause));
          // Reveal the overlay so the error is visible and Esc can dismiss it.
          void ipc.screenshotReady().catch(() => undefined);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [t]);

  const cancel = useCallback(() => {
    void ipc.cancelRegionCapture().catch(() => undefined);
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        cancel();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [cancel]);

  const localPoint = (clientX: number, clientY: number) => {
    const rect = rootRef.current?.getBoundingClientRect();
    return { x: clientX - (rect?.left ?? 0), y: clientY - (rect?.top ?? 0) };
  };

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0 || busy || error || !ready || !session) return;
    const point = localPoint(event.clientX, event.clientY);
    dragStart.current = point;
    setSelection({ left: point.x, top: point.y, width: 0, height: 0 });
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    const start = dragStart.current;
    if (!start || busy) return;
    const point = localPoint(event.clientX, event.clientY);
    setSelection({
      left: Math.min(start.x, point.x),
      top: Math.min(start.y, point.y),
      width: Math.abs(point.x - start.x),
      height: Math.abs(point.y - start.y),
    });
  };

  const onPointerUp = (event: PointerEvent<HTMLDivElement>) => {
    const start = dragStart.current;
    if (!start || busy || !session) return;
    dragStart.current = null;
    const point = localPoint(event.clientX, event.clientY);
    const box: SelectionBox = {
      left: Math.min(start.x, point.x),
      top: Math.min(start.y, point.y),
      width: Math.abs(point.x - start.x),
      height: Math.abs(point.y - start.y),
    };
    setSelection(box);
    if (box.width < 2 || box.height < 2) {
      setSelection(null);
      return;
    }
    const view = viewSize();
    if (!view) return;
    // The selection belongs to this overlay's display, so it maps into that display's
    // own capture instead of a canvas shared by displays of different densities.
    const pixels = selectionToPixels(box, session.width, session.height, view.width, view.height);
    if (!pixels) {
      setSelection(null);
      return;
    }
    setBusy(true);
    void ipc
      .captureRegion(pixels.x, pixels.y, pixels.width, pixels.height)
      .catch((cause) => {
        setBusy(false);
        setError(errorText(t, cause));
      });
  };

  const view = viewSize();
  const hint =
    session && view && selection
      ? selectionToPixels(selection, session.width, session.height, view.width, view.height)
      : null;
  const imageUrl = session ? convertFileSrc(session.imagePath) : null;
  const hasSelection = Boolean(selection && selection.width > 0 && selection.height > 0);

  return (
    <div
      className="shot-root"
      ref={rootRef}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={() => {
        dragStart.current = null;
        setSelection(null);
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        cancel();
      }}
    >
      {imageUrl && (
        <img
          className="shot-image"
          src={imageUrl}
          alt=""
          draggable={false}
          onLoad={() => {
            setReady(true);
            void ipc.screenshotReady().catch(() => undefined);
          }}
          onError={() => {
            setError(t('screenshotFailed'));
            void ipc.screenshotReady().catch(() => undefined);
          }}
        />
      )}

      {hasSelection && selection ? (
        <>
          <div className="shot-dim" style={{ left: 0, top: 0, right: 0, height: selection.top }} />
          <div
            className="shot-dim"
            style={{ left: 0, top: selection.top + selection.height, right: 0, bottom: 0 }}
          />
          <div
            className="shot-dim"
            style={{
              left: 0,
              top: selection.top,
              width: selection.left,
              height: selection.height,
            }}
          />
          <div
            className="shot-dim"
            style={{
              left: selection.left + selection.width,
              top: selection.top,
              right: 0,
              height: selection.height,
            }}
          />
          <div
            className="shot-selection"
            style={{
              left: selection.left,
              top: selection.top,
              width: selection.width,
              height: selection.height,
            }}
          />
          <div
            className="shot-size"
            style={{
              left: selection.left + selection.width,
              top: selection.top + selection.height + 8,
            }}
          >
            {hint ? `${hint.width} × ${hint.height}` : ''}
          </div>
        </>
      ) : (
        <div className="shot-dim shot-dim-full" />
      )}

      {!error && !busy && (
        <div className="shot-hint">
          {ready ? t('screenshotHint') : t('screenshotPreparing')}
        </div>
      )}
      {busy && <div className="shot-hint">{t('screenshotCopying')}</div>}
      {error && (
        <div className="shot-error" onPointerDown={(event) => event.stopPropagation()}>
          <p>{error}</p>
          <button type="button" onClick={cancel}>
            {t('close')}
          </button>
        </div>
      )}
    </div>
  );
}
