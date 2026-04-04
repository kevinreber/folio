import { useEffect, type ReactNode } from "react";

export function Modal({
  open,
  onClose,
  children,
  maxWidth,
}: {
  open: boolean;
  onClose: () => void;
  children: ReactNode;
  maxWidth?: string;
}) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    if (open) document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="modal">
      <div className="modal-backdrop" onClick={onClose} />
      <div
        className="modal-content"
        style={maxWidth ? { maxWidth } : undefined}
      >
        <button className="modal-close" onClick={onClose}>
          &times;
        </button>
        {children}
      </div>
    </div>
  );
}
