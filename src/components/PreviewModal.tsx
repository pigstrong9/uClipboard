import { convertFileSrc } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

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
  const extra = parseExtra(item.extra);
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

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div onClick={(event) => event.stopPropagation()}>
        {item.kind === 'image' && (
          <figure className="preview-image">
            <img
              src={imageUrl ?? item.thumb_data_url ?? ''}
              alt={t('previewImageAlt')}
            />
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
            <h3>{t('filesCount', { n: parseFiles(item.content).length })}</h3>
            <pre>{parseFiles(item.content).join('\n')}</pre>
          </div>
        )}
      </div>
      <button type="button" className="icon-btn preview-close" title={t('close')} onClick={onClose}>
        <Icon name="close" />
      </button>
    </div>
  );
}
