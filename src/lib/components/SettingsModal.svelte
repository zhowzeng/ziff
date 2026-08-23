<script module>
  export const DEFAULT_SETTINGS = {
    theme: 'system',
    language: 'zh-Hant',
    zoom: 100,
    diffFontSize: 'md',
    defaultDiffMode: 'unstaged',
    usePrefixPrompt: false,
    prefixPrompt: '',
  };

  /** @typedef {typeof DEFAULT_SETTINGS} Settings */
</script>

<script>
  import Modal from './Modal.svelte';
  import Segmented from './Segmented.svelte';
  import Switch from './Switch.svelte';
  import Dropdown from './Dropdown.svelte';
  import Textarea from './Textarea.svelte';
  import Button from './Button.svelte';

  const THEMES = [
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' },
    { value: 'system', label: 'System' },
  ];
  const LANGUAGES = [
    { value: 'zh-Hant', label: '繁體中文' },
    { value: 'en', label: 'English' },
    { value: 'ja', label: '日本語' },
  ];
  const DIFF_FONT_SIZES = [
    { value: 'sm', label: 'Small' },
    { value: 'md', label: 'Medium' },
    { value: 'lg', label: 'Large' },
  ];
  const DEFAULT_DIFF_MODES = [
    { value: 'unstaged', label: 'Unstaged' },
    { value: 'branch', label: 'Branch' },
    { value: 'staged', label: 'Staged' },
  ];

  /**
   * @typedef {Object} Props
   * @property {boolean} [open]
   * @property {() => void} [onClose]
   * @property {Settings} settings
   * @property {(settings: Settings) => void} onChange
   */

  /** @type {Props} */
  let { open = false, onClose, settings, onChange } = $props();

  /** @param {Partial<Settings>} patch */
  function set(patch) {
    onChange({ ...settings, ...patch });
  }

  /** @param {number} delta */
  function stepZoom(delta) {
    set({ zoom: Math.min(150, Math.max(75, settings.zoom + delta)) });
  }
</script>

{#snippet footer()}
  <Button variant="primary" size="sm" onclick={onClose}>完成</Button>
{/snippet}

<Modal {open} {onClose} title="設定" width={480} {footer}>
  <div class="section">
    <div class="section-title">外觀</div>
    <div class="row">
      <span class="row-label">主題</span>
      <Segmented value={settings.theme} onChange={(v) => set({ theme: v })} options={THEMES} />
    </div>
    <div class="row">
      <span class="row-label">語言</span>
      <Dropdown label="Language" options={LANGUAGES} value={settings.language} onChange={(v) => set({ language: v })} width={160} />
    </div>
  </div>

  <div class="section">
    <div class="section-title">檢視</div>
    <div class="row">
      <span class="row-label">縮放</span>
      <div class="zoom-stepper">
        <button class="zoom-btn" onclick={() => stepZoom(-10)}>−</button>
        <span class="zoom-value">{settings.zoom}%</span>
        <button class="zoom-btn" onclick={() => stepZoom(10)}>+</button>
      </div>
    </div>
    <div class="row">
      <span class="row-label">Diff 字級</span>
      <Segmented value={settings.diffFontSize} onChange={(v) => set({ diffFontSize: v })} options={DIFF_FONT_SIZES} />
    </div>
    <div class="row">
      <span class="row-label">預設 Diff 模式</span>
      <Segmented value={settings.defaultDiffMode} onChange={(v) => set({ defaultDiffMode: v })} options={DEFAULT_DIFF_MODES} />
    </div>
  </div>

  <div class="section last">
    <div class="section-title">Copy to Agent</div>
    <div class="row">
      <span class="row-label">加上 prefix prompt</span>
      <Switch checked={settings.usePrefixPrompt} onchange={(v) => set({ usePrefixPrompt: v })} />
    </div>
    {#if settings.usePrefixPrompt}
      <Textarea
        rows={3}
        value={settings.prefixPrompt}
        oninput={(e) => set({ prefixPrompt: e.currentTarget.value })}
        placeholder="例如：You are reviewing a Rust codebase. Be concise and cite file paths."
      />
    {/if}
  </div>
</Modal>

<style>
  .section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px 0;
    border-bottom: 1px solid var(--border-default);
  }
  .section:first-child {
    padding-top: 0;
  }
  .section.last {
    border-bottom: none;
    padding-bottom: 0;
  }
  .section-title {
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .row-label {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--text-primary);
  }
  .zoom-stepper {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .zoom-btn {
    width: 26px;
    height: 26px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-default);
    background: var(--gray-0);
    cursor: pointer;
    font-size: 15px;
    line-height: 1;
    color: var(--text-secondary);
  }
  .zoom-btn:hover {
    background: var(--bg-subtle);
  }
  .zoom-value {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    width: 40px;
    text-align: center;
  }
</style>
