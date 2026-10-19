import { AlertCircle, CheckCircle2, Info, X } from "lucide-react";
import { useAppStore } from "@/state/store";
import { IconButton } from "./IconButton";

export function Toasts() {
  const toasts = useAppStore((s) => s.toasts);
  const dismiss = useAppStore((s) => s.dismissToast);
  return (
    <div className="toast-stack" aria-live="polite">
      {toasts.map((toast) => (
        <div key={toast.id} className={`toast toast-${toast.kind}`}>
          <span className="toast-icon">
            {toast.kind === "error" ? (
              <AlertCircle size={18} />
            ) : toast.kind === "success" ? (
              <CheckCircle2 size={18} />
            ) : (
              <Info size={18} />
            )}
          </span>
          <div className="toast-text">
            <div className="toast-message">{toast.message}</div>
            {toast.detail && <div className="toast-detail">{toast.detail}</div>}
          </div>
          <IconButton label="Dismiss" size="sm" onClick={() => dismiss(toast.id)}>
            <X size={14} />
          </IconButton>
        </div>
      ))}
    </div>
  );
}
