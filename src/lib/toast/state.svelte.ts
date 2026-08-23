export type ToastVariant = 'default' | 'success' | 'danger' | 'warning';

export type ToastItem = {
  id: number;
  message: string;
  variant: ToastVariant;
};

let toasts = $state<ToastItem[]>([]);
let nextId = 0;

export function getToasts() {
  return toasts;
}

export function toast(
  message: string,
  { variant = 'default', duration = 4000 }: { variant?: ToastVariant; duration?: number } = {}
) {
  const id = nextId++;
  toasts.push({ id, message, variant });
  if (duration > 0) {
    setTimeout(() => dismiss(id), duration);
  }
  return id;
}

export function dismiss(id: number) {
  const i = toasts.findIndex((t) => t.id === id);
  if (i !== -1) toasts.splice(i, 1);
}
