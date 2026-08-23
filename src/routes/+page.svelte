<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Button from "$lib/components/Button.svelte";
  import Input from "$lib/components/Input.svelte";
  import Badge from "$lib/components/Badge.svelte";
  import IconButton from "$lib/components/IconButton.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import Checkbox from "$lib/components/Checkbox.svelte";
  import Switch from "$lib/components/Switch.svelte";
  import Tooltip from "$lib/components/Tooltip.svelte";
  import { toast } from "$lib/toast/state.svelte";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }

  let modalOpen = $state(false);
  let autoSync = $state(true);
  let notifyOnMention = $state(false);
</script>

<main class="page">
  <header class="page-header">
    <div class="brand">
      <Icon name="git-pull-request" size={20} color="var(--accent)" />
      <span class="wordmark">Ziff</span>
    </div>
    <Badge variant="accent">Design system online</Badge>
  </header>

  <a class="preview-link" href="/review">
    <Icon name="arrow-right" size={14} />
    Open the diff review screen preview
  </a>

  <section class="card">
    <h1>Welcome to Ziff</h1>
    <p class="muted">
      Tokens and core components are now wired up from the Ziff design system
      (Mona Sans, GitHub Primer colors, teal accent).
    </p>

    <form class="row" onsubmit={greet}>
      <Input bind:value={name} placeholder="Enter a name..." />
      <Button type="submit" variant="primary">Greet</Button>
    </form>
    {#if greetMsg}
      <p class="result">{greetMsg}</p>
    {/if}

    <div class="row" style="margin-top: var(--space-6); gap: var(--space-2)">
      <Button variant="primary">Sync pull request</Button>
      <Button variant="secondary">Resolve comment</Button>
      <Button variant="ghost">Cancel</Button>
      <Button variant="danger">Delete branch</Button>
      <IconButton icon="settings" title="Settings" />
    </div>

    <div class="row" style="margin-top: var(--space-4); gap: var(--space-2)">
      <Badge variant="neutral">Draft</Badge>
      <Badge variant="success">Merged</Badge>
      <Badge variant="danger">Closed</Badge>
      <Badge variant="outline">Open</Badge>
    </div>
  </section>

  <section class="card" style="margin-top: var(--space-6)">
    <h2 class="section-title">New components</h2>
    <p class="muted">Modal, Toast, Checkbox, Switch, Tooltip — Svelte-only additions, no screen using them yet.</p>

    <div class="row" style="gap: var(--space-3); flex-wrap: wrap">
      <Button variant="secondary" onclick={() => (modalOpen = true)}>Open modal</Button>
      <Button variant="secondary" onclick={() => toast('Synced 2m ago')}>Toast — default</Button>
      <Button variant="secondary" onclick={() => toast('Comment saved to queue', { variant: 'success' })}>Toast — success</Button>
      <Button variant="secondary" onclick={() => toast('Sync failed — check connection', { variant: 'danger' })}>Toast — danger</Button>
      <Tooltip text="Copies all saved comments for your CLI agent">
        <IconButton icon="info" title="What does this do?" />
      </Tooltip>
    </div>

    <div class="row" style="margin-top: var(--space-5); gap: var(--space-5)">
      <Checkbox bind:checked={autoSync} label="Auto-sync on file save" />
      <Switch bind:checked={notifyOnMention} label="Notify on @mention" />
    </div>
  </section>
</main>

<Modal open={modalOpen} onClose={() => (modalOpen = false)} title="Repository settings" width={420}>
  <p class="muted" style="margin: 0 0 var(--space-4)">Settings scope isn't defined yet — this is a placeholder to show the Modal component.</p>
  <Checkbox bind:checked={autoSync} label="Auto-sync on file save" />
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (modalOpen = false)}>Cancel</Button>
    <Button variant="primary" onclick={() => (modalOpen = false)}>Save</Button>
  {/snippet}
</Modal>

<style>
  .page {
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-8) var(--space-4);
    font-family: var(--font-sans);
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-6);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .wordmark {
    font-size: var(--text-lg);
    font-weight: 700;
    color: var(--text-primary);
  }

  .card {
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    box-shadow: var(--shadow-sm);
  }

  h1 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-2xl);
    font-weight: 700;
  }

  .section-title {
    margin: 0 0 var(--space-2);
    font-size: var(--text-lg);
    font-weight: 700;
  }

  .muted {
    color: var(--text-secondary);
    font-size: var(--text-sm);
    margin: 0 0 var(--space-4);
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .preview-link {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    margin-bottom: var(--space-4);
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-link);
    text-decoration: none;
  }

  .preview-link:hover {
    color: var(--text-link-hover);
  }

  .result {
    margin-top: var(--space-3);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }
</style>
