<script>
  import Modal from './Modal.svelte';
  import Segmented from './Segmented.svelte';
  import Switch from './Switch.svelte';
  import Textarea from './Textarea.svelte';
  import Button from './Button.svelte';

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
   * @typedef {import('$lib/settings/state.svelte').Settings} Settings
   * @typedef {import('$lib/settings/state.svelte').DiffFontSize} DiffFontSize
   * @typedef {import('$lib/settings/state.svelte').DefaultDiffMode} DefaultDiffMode
   */

  /**
   * @typedef {Object} Props
   * @property {boolean} [open]
   * @property {() => void} [onClose]
   * @property {Settings} settings
   * @property {(patch: Partial<Settings>) => void} onChange
   */

  /** @type {Props} */
  let { open = false, onClose, settings, onChange } = $props();
</script>

{#snippet footer()}
  <Button variant="primary" size="sm" onclick={onClose}>完成</Button>
{/snippet}

<Modal {open} {onClose} title="設定" width={480} {footer}>
  <div class="section">
    <div class="section-title">檢視</div>
    <div class="row">
      <span class="row-label">Diff 字級</span>
      <Segmented value={settings.diffFontSize} onChange={(v) => onChange({ diffFontSize: /** @type {DiffFontSize} */ (v) })} options={DIFF_FONT_SIZES} />
    </div>
    <div class="row">
      <span class="row-label">預設 Diff 模式</span>
      <Segmented value={settings.defaultDiffMode} onChange={(v) => onChange({ defaultDiffMode: /** @type {DefaultDiffMode} */ (v) })} options={DEFAULT_DIFF_MODES} />
    </div>
    <div class="row">
      <span class="row-label">長行自動換行</span>
      <Switch checked={settings.lineWrap} onchange={(v) => onChange({ lineWrap: v })} />
    </div>
  </div>

  <div class="section last">
    <div class="section-title">Copy to Agent</div>
    <div class="row">
      <span class="row-label">加上 prefix prompt</span>
      <Switch checked={settings.usePrefixPrompt} onchange={(v) => onChange({ usePrefixPrompt: v })} />
    </div>
    {#if settings.usePrefixPrompt}
      <Textarea
        rows={3}
        value={settings.prefixPrompt}
        oninput={(e) => onChange({ prefixPrompt: e.currentTarget.value })}
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
</style>
