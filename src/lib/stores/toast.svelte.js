let toasts = $state([]);
let nextId = 0;

export function getToasts() {
  return toasts;
}

export function toast(message, { variant = 'default', duration = 4000 } = {}) {
  const id = nextId++;
  toasts.push({ id, message, variant });
  if (duration > 0) {
    setTimeout(() => dismiss(id), duration);
  }
  return id;
}

export function dismiss(id) {
  const i = toasts.findIndex((t) => t.id === id);
  if (i !== -1) toasts.splice(i, 1);
}
