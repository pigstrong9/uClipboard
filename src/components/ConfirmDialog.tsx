import { useI18n } from '../lib/useI18n';

interface Props {
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({ message, onConfirm, onCancel }: Props) {
  const { t } = useI18n();
  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <div className="confirm-dialog" onClick={(event) => event.stopPropagation()}>
        <p className="confirm-message">{message}</p>
        <div className="confirm-actions">
          <button type="button" className="settings-button" onClick={onCancel}>
            {t('cancel')}
          </button>
          <button type="button" className="settings-button danger" onClick={onConfirm}>
            {t('confirm')}
          </button>
        </div>
      </div>
    </div>
  );
}
