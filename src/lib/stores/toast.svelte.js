/**
 * @typedef {Object} ToastItem
 * @property {number} id
 * @property {string} message
 * @property {'default'|'success'|'danger'|'warning'} variant
 */

/** @type {ToastItem[]} */
let toasts = $state([]);
let nextId = 0;

export function getToasts() {
  return toasts;
}

/**
 * @param {string} message
 * @param {{ variant?: 'default'|'success'|'danger'|'warning', duration?: number }} [options]
 */
export function toast(message, { variant = 'default', duration = 4000 } = {}) {
  const id = nextId++;
  toasts.push({ id, message, variant });
  if (duration > 0) {
    setTimeout(() => dismiss(id), duration);
  }
  return id;
}

/** @param {number} id */
export function dismiss(id) {
  const i = toasts.findIndex((t) => t.id === id);
  if (i !== -1) toasts.splice(i, 1);
}
