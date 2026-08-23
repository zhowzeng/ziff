<script>
  let { label } = $props();
</script>

<div
  style="background:var(--diff-hunk-bg);color:var(--diff-hunk-text);font-family:var(--font-mono);font-size:var(--text-xs);padding:4px 12px;border-top:1px solid var(--border-muted);border-bottom:1px solid var(--border-muted)"
>
  {label}
</div>
