// The state modules are written against the WebView, where localStorage always exists
// (docs/decisions/0006). Node has none, so importing settings — directly or through
// anything that reads it — would throw before a single assertion ran. This is that
// storage: in-memory, one per test file, and writable by tests that care what was
// stored before the app loaded.
import { vi } from 'vitest';

const store = new Map<string, string>();

vi.stubGlobal('localStorage', {
  getItem: (key: string) => store.get(key) ?? null,
  setItem: (key: string, value: string) => void store.set(key, String(value)),
  removeItem: (key: string) => void store.delete(key),
  clear: () => store.clear(),
});
