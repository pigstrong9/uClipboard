import { useEffect, useState } from 'react';

import * as ipc from '../lib/ipc';
import { Icon } from '../lib/icons';
import { parseExtra, parseFiles, type ClipItem } from '../lib/types';

interface Props {
  item: ClipItem;
  onClose: () => void;
}

export function PreviewModal({ item, onClose }: Props) {
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const extra = parseExtra(item.extra);

  useEffect(() => {
    if (item.kind !== 'image') return;
    let cancelled = false;
    ipc
      .imageDataUrl(item.id)
      .then((url) => {
        if (!cancelled) setImageUrl(url);
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
              alt="剪贴板图片"
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
            <h3>{parseFiles(item.content).length} 个文件</h3>
            <pre>{parseFiles(item.content).join('\n')}</pre>
          </div>
        )}
      </div>
      <button type="button" className="icon-btn preview-close" title="关闭" onClick={onClose}>
        <Icon name="close" />
      </button>
    </div>
  );
}
