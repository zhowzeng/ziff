export type ToastVariant = 'default' | 'success' | 'danger' | 'warning';

// A single button in the toast, for an action only worth offering while the toast that
// reports what happened is still on screen — undoing it (docs/decisions/0014).
export type ToastAction = {
  label: string;
  onclick: () => void;
};

export type ToastItem = {
  id: number;
  message: string;
  variant: ToastVariant;
  action?: ToastAction;
};

let toasts = $state<ToastItem[]>([]);
let nextId = 0;

export function getToasts() {
  return toasts;
}

export function toast(
  message: string,
  {
    variant = 'default',
    duration = 4000,
    action,
  }: { variant?: ToastVariant; duration?: number; action?: ToastAction } = {}
) {
  const id = nextId++;
  toasts.push({ id, message, variant, action });
  if (duration > 0) {
    setTimeout(() => dismiss(id), duration);
  }
  return id;
}

export function dismiss(id: number) {
  const i = toasts.findIndex((t) => t.id === id);
  if (i !== -1) toasts.splice(i, 1);
}
