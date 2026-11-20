import { useUi } from '../stores/ui';

export function Toast() {
  const visible = useUi((state) => state.visible);
  const message = useUi((state) => state.message);
  if (!visible) return null;
  return (
    <div className="toast" role="status">
      {message}
    </div>
  );
}
