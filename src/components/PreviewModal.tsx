import { convertFileSrc } from '@tauri-apps/api/core';
import { useEffect, useRef, useState } from 'react';

import { plural } from '../lib/format';
import * as ipc from '../lib/ipc';
import { Icon } from '../lib/icons';
import { useI18n } from '../lib/useI18n';
import { parseExtra, parseFiles, type ClipItem } from '../lib/types';

interface Props {
  item: ClipItem;
  onClose: () => void;
}

export function PreviewModal({ item, onClose }: Props) {
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  const extra = parseExtra(item.extra);
  const thumbUrl = item.thumb_path ? convertFileSrc(item.thumb_path) : null;
  const previewUrl = imageUrl ?? thumbUrl;
  const { t } = useI18n();

  useEffect(() => {
    if (item.kind !== 'image') return;
    let cancelled = false;
    ipc
      .imageFilePath(item.id)
      .then((path) => {
        if (!cancelled) setImageUrl(convertFileSrc(path));
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [item.id, item.kind]);

  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    closeRef.current?.focus();

    const trapTab = (event: KeyboardEvent) => {
      if (event.key !== 'Tab') return;
      const nodes = dialogRef.current?.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      );
      if (!nodes || nodes.length === 0) return;
      const first = nodes[0];
      const last = nodes[nodes.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };

    window.addEventListener('keydown', trapTab);
    return () => {
      window.removeEventListener('keydown', trapTab);
      previous?.focus?.();
    };
  }, []);

  return (
    <div
      className="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label={t('previewImageAlt')}
      ref={dialogRef}
      onClick={onClose}
    >
      <div onClick={(event) => event.stopPropagation()}>
        {item.kind === 'image' && (
          <figure className="preview-image">
            {previewUrl ? (
              <img src={previewUrl} alt={t('previewImageAlt')} />
            ) : (
              <div className="thumb-placeholder">{t('image')}</div>
            )}
            <figcaption>
              {extra.width ?? 0} × {extra.height ?? 0}
            </figcaption>
          </figure>
        )}
        {item.kind === 'text' && (
          <div className="preview-text">
            <pre>{item.content}</pre>
          </div>
        )}
        {item.kind === 'files' && (
          <div className="preview-text">
            <h3>{plural(t, 'filesCount', parseFiles(item.content).length)}</h3>
            <pre>{parseFiles(item.content).join('\n')}</pre>
          </div>
        )}
      </div>
      <button
        type="button"
        className="icon-btn preview-close"
        title={t('close')}
        aria-label={t('close')}
        ref={closeRef}
        onClick={onClose}
      >
        <Icon name="close" />
      </button>
    </div>
  );
}
