// Keyboard shortcuts the review screen promises on its own buttons, so the label
// printed on the button and the keys that actually fire it can't drift apart.

const isMac = /Mac/.test(navigator.platform);

// Copy the whole Comment Queue for the CLI agent. Handled at page level rather than
// inside the drawer: the drawer is only rendered while it's open, and the shortcut has
// to keep working with it closed.
export const copyAllShortcut = {
  label: isMac ? '⌘⇧C' : 'Ctrl+Shift+C',
  matches(e: KeyboardEvent) {
    return (isMac ? e.metaKey : e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 'c';
  },
};

// Re-read the diff after the CLI agent has edited the files it was handed. Handled at
// page level for the same reason as the one above, and because the keystroke has to be
// swallowed even when there is nothing to reload: letting the webview take it as a page
// reload would throw away the Comment Queue, which only lives in memory
// (docs/decisions/0006).
export const refreshShortcut = {
  label: isMac ? '⌘R' : 'Ctrl+R',
  matches(e: KeyboardEvent) {
    return (isMac ? e.metaKey : e.ctrlKey) && !e.shiftKey && e.key.toLowerCase() === 'r';
  },
};
