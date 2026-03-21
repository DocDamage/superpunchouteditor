import { useEffect } from 'react';
import { useUiStore } from '../store/uiStore';
import type { Toast } from '../store/types';

function ToastItem({ toast }: { toast: Toast }) {
  const removeToast = useUiStore((s) => s.removeToast);

  return (
    <div
      className={`toast toast--${toast.type}`}
      role="alert"
      onClick={() => removeToast(toast.id)}
      title="Click to dismiss"
    >
      <span className="toast__message">{toast.message}</span>
      <button
        className="toast__close"
        onClick={(e) => { e.stopPropagation(); removeToast(toast.id); }}
        aria-label="Dismiss"
      >
        ×
      </button>
    </div>
  );
}

export function ToastContainer() {
  const toasts = useUiStore((s) => s.toasts);

  return (
    <div className="toast-container" aria-live="polite" aria-atomic="false">
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} />
      ))}
    </div>
  );
}

/**
 * Call this instead of alert() anywhere in the app.
 * Convenience wrapper so components don't need to import the store directly.
 */
export function showToast(
  message: string,
  type: Toast['type'] = 'info',
  duration = 5000,
) {
  useUiStore.getState().addToast(message, type, duration);
}
