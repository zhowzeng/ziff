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
