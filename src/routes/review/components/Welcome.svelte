<script lang="ts">
  import Button from "$lib/components/Button.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import Logo from "$lib/components/Logo.svelte";
  import { copyAllShortcut, nextFileShortcut, prevFileShortcut, refreshShortcut } from "../shortcuts";

  interface Props {
    onAddRepo: () => void;
  }
  let { onAddRepo }: Props = $props();

  // A decorative diff that "types itself", so the very first screen shows what the app
  // is for: read a change, anchor a note to it, hand it to an agent.
  const DEMO = [
    { kind: "ctx", no: 12, code: "const token = sign({ sub: user.id });" },
    { kind: "del", no: 13, code: "return { token };" },
    { kind: "add", no: 13, code: "return { token, expires };" },
  ] as const;

  const KEYS = [
    { keys: [nextFileShortcut.label, prevFileShortcut.label], label: "Next / previous file" },
    { keys: [refreshShortcut.label], label: "Re-read the diff" },
    { keys: [copyAllShortcut.label], label: "Copy queue for agent" },
  ];
</script>

<div class="welcome">

  <div class="stack">
    <div class="mark"><Logo size={56} /></div>
    <h1>Review the diff.<br /><em>Hand it to your agent.</em></h1>
    <p>Read local changes, pin a note to any line, and copy the whole queue to Codex or Claude in one keystroke.</p>

    <div class="cta">
      <Button variant="primary" size="lg" onclick={onAddRepo}>
        <Icon name="folder-plus" size={15} color="var(--accent-fg)" />
        Add a repo
      </Button>
    </div>

    <div class="demo" aria-hidden="true">
      <div class="demo-bar"><span></span><span></span><span></span><b>session.ts</b></div>
      {#each DEMO as l, i (i)}
        <div class="dl {l.kind}" style:--d="{0.35 + i * 0.28}s">
          <span class="n">{l.no}</span>
          <span class="s">{l.kind === "add" ? "+" : l.kind === "del" ? "−" : ""}</span>
          <span class="c">{l.code}</span>
          {#if l.kind === "add"}<span class="note"><Icon name="message-square" size={11} color="var(--accent-emphasis)" />expires in UTC?</span>{/if}
        </div>
      {/each}
    </div>

    <ul class="keys">
      {#each KEYS as k (k.label)}
        <li>
          <span class="kbds">{#each k.keys as key (key)}<kbd>{key}</kbd>{/each}</span>
          <span>{k.label}</span>
        </li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .welcome {
    position: relative;
    flex: 1;
    display: grid;
    place-items: center;
    overflow: hidden;
    padding: 32px;
  }
  .stack {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    max-width: 640px;
  }
  .stack > * {
    animation: z-fade-up var(--dur-slow) var(--ease-out) both;
  }
  h1 {
    margin: 22px 0 0;
    font-family: var(--font-display);
    font-size: 52px;
    line-height: 1.08;
    font-weight: 400;
    letter-spacing: -0.015em;
    color: var(--text-primary);
    animation-delay: 60ms;
  }
  h1 em {
    font-style: normal;
    color: var(--text-secondary);
  }
  p {
    margin: 14px 0 0;
    max-width: 420px;
    font-family: var(--font-sans);
    font-size: var(--text-base);
    line-height: var(--leading-relaxed);
    color: var(--text-secondary);
    animation-delay: 120ms;
  }
  .cta {
    margin-top: 24px;
    animation-delay: 180ms;
  }

  .demo {
    margin-top: 36px;
    width: 440px;
    max-width: 100%;
    text-align: left;
    border-radius: var(--radius-lg);
    background: var(--bg-raised);
    box-shadow: var(--shadow-lg), 0 0 0 1px var(--border-default);
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    animation-delay: 260ms;
  }
  .demo-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 9px 12px;
    background: var(--bg-subtle);
    border-bottom: 1px solid var(--border-muted);
  }
  .demo-bar span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--border-strong);
  }
  .demo-bar b {
    margin-left: 8px;
    font-weight: 500;
    color: var(--text-tertiary);
  }
  .dl {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    height: 26px;
    background: var(--dl-bg, transparent);
    box-shadow: inset 2px 0 0 var(--dl-bar, transparent);
    animation: z-fade-up var(--dur-slow) var(--ease-out) both;
    animation-delay: calc(var(--d) + 0.3s);
  }
  .dl.add { --dl-bg: var(--diff-add-bg); --dl-bar: var(--diff-add-border); --dl-sign: var(--diff-add-text); }
  .dl.del { --dl-bg: var(--diff-remove-bg); --dl-bar: var(--diff-remove-border); --dl-sign: var(--diff-remove-text); }
  .n { width: 20px; text-align: right; color: var(--text-tertiary); }
  .s { width: 10px; color: var(--dl-sign, transparent); }
  .c { color: var(--text-primary); white-space: pre; }
  .note {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    background: var(--accent-subtle);
    box-shadow: inset 0 0 0 1px var(--accent-muted-border);
    color: var(--accent-emphasis);
    font-family: var(--font-sans);
    font-size: 11px;
    white-space: nowrap;
  }

  .keys {
    margin: 28px 0 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px 22px;
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    animation-delay: 340ms;
  }
  .keys li {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .kbds {
    display: inline-flex;
    gap: 3px;
  }
  kbd {
    min-width: 20px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-raised);
    box-shadow: 0 1px 0 var(--border-strong), inset 0 0 0 1px var(--border-default);
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    text-align: center;
  }
</style>
