<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Button from "$lib/components/Button.svelte";
  import Input from "$lib/components/Input.svelte";
  import Badge from "$lib/components/Badge.svelte";
  import IconButton from "$lib/components/IconButton.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="page">
  <header class="page-header">
    <div class="brand">
      <Icon name="git-pull-request" size={20} color="var(--accent)" />
      <span class="wordmark">Ziff</span>
    </div>
    <Badge variant="accent">Design system online</Badge>
  </header>

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
</main>

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

  .result {
    margin-top: var(--space-3);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }
</style>
