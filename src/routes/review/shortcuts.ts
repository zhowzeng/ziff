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

// The navigation keys below are bare letters, so they must stay out of the way while
// the reviewer is typing — a comment in the thread's Textarea, or a name in the file
// filter. A focused checkbox or button isn't typing, and swallowing the key there would
// only make the shortcut look broken after a click.
function isTyping(e: KeyboardEvent) {
  const t = e.target;
  if (!(t instanceof HTMLElement)) return false;
  if (t.isContentEditable || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT') return true;
  return t instanceof HTMLInputElement && !['checkbox', 'radio', 'button', 'submit'].includes(t.type);
}

// Any modifier means the keystroke is meant for something else (⌘J, a browser binding),
// and Shift turns `j` into `J`, which these don't claim either.
function bareKey(key: string) {
  return {
    label: key,
    matches(e: KeyboardEvent) {
      return e.key === key && !e.metaKey && !e.ctrlKey && !e.altKey && !isTyping(e);
    },
  };
}

// File and hunk navigation, after GitHub's j / k and Vim's n / p muscle memory.
// j / k move through the changed files in sidebar order; handled at page level, which
// owns the tree. n / p jump between hunks of the open diff; handled in DiffPanel, which
// owns the scroll position.
export const nextFileShortcut = bareKey('j');
export const prevFileShortcut = bareKey('k');
export const nextHunkShortcut = bareKey('n');
export const prevHunkShortcut = bareKey('p');
