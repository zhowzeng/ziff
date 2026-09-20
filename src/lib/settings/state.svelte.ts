// App settings. They live in the WebView's own localStorage with no Rust command
// behind them (docs/decisions/0006).

export type DiffFontSize = 'sm' | 'md' | 'lg';
export type DefaultDiffMode = 'unstaged' | 'staged' | 'branch';

export interface Settings {
  diffFontSize: DiffFontSize;
  defaultDiffMode: DefaultDiffMode;
  usePrefixPrompt: boolean;
  prefixPrompt: string;
}

const DEFAULT_SETTINGS: Settings = {
  diffFontSize: 'md',
  defaultDiffMode: 'unstaged',
  usePrefixPrompt: false,
  prefixPrompt: '',
};

const STORAGE_KEY = 'ziff.settings';

const DIFF_FONT_SIZES: DiffFontSize[] = ['sm', 'md', 'lg'];
const DEFAULT_DIFF_MODES: DefaultDiffMode[] = ['unstaged', 'staged', 'branch'];

/** What each `diffFontSize` means in the diff and File View gutters. */
export const DIFF_FONT_SIZE_PX: Record<DiffFontSize, string> = {
  sm: '12px',
  md: '13px',
  lg: '15px',
};

// Stored settings are whatever an older build — or a hand-edited localStorage — left
// behind, so every field is checked before it reaches the UI or a Rust command.
function sanitize(stored: unknown): Settings {
  if (typeof stored !== 'object' || stored === null) return { ...DEFAULT_SETTINGS };
  const s = stored as Record<string, unknown>;
  return {
    diffFontSize: DIFF_FONT_SIZES.includes(s.diffFontSize as DiffFontSize)
      ? (s.diffFontSize as DiffFontSize)
      : DEFAULT_SETTINGS.diffFontSize,
    defaultDiffMode: DEFAULT_DIFF_MODES.includes(s.defaultDiffMode as DefaultDiffMode)
      ? (s.defaultDiffMode as DefaultDiffMode)
      : DEFAULT_SETTINGS.defaultDiffMode,
    usePrefixPrompt:
      typeof s.usePrefixPrompt === 'boolean' ? s.usePrefixPrompt : DEFAULT_SETTINGS.usePrefixPrompt,
    prefixPrompt: typeof s.prefixPrompt === 'string' ? s.prefixPrompt : DEFAULT_SETTINGS.prefixPrompt,
  };
}

function load(): Settings {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) return { ...DEFAULT_SETTINGS };
  try {
    return sanitize(JSON.parse(raw));
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export const settings = $state<Settings>(load());

export function updateSettings(patch: Partial<Settings>) {
  Object.assign(settings, patch);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}
