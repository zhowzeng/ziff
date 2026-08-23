<script>
  const colors = ['#0ea5a4', '#2563eb', '#7c3aed', '#d97706', '#dc2626', '#059669'];

  let { name, src, size = 28, source } = $props();

  let initial = $derived((name || '?').slice(0, 1).toUpperCase());
  let color = $derived(colors[(name || '').charCodeAt(0) % colors.length] || colors[0]);
</script>

{#if src}
  <img
    {src}
    width={size}
    height={size}
    style="border-radius:50%;object-fit:cover"
    alt={name}
  />
{:else if source !== 'github'}
  <div
    style={`width:${size}px;height:${size}px;border-radius:50%;background:${color};color:#fff;
      display:flex;align-items:center;justify-content:center;flex-shrink:0;
      font-family:var(--font-sans);font-weight:600;font-size:${size * 0.42}px`}
  >
    {initial}
  </div>
{:else}
  <div
    style={`width:${size}px;height:${size}px;border-radius:50%;background:#24292f;color:#fff;
      display:flex;align-items:center;justify-content:center;flex-shrink:0`}
  >
    <svg width={size * 0.62} height={size * 0.62} viewBox="0 0 24 24" fill="#fff" role="img" aria-label={name}>
      <path
        d="M12 .5C5.73.5.5 5.73.5 12c0 5.08 3.29 9.39 7.86 10.91.57.1.78-.25.78-.55 0-.27-.01-1.17-.02-2.12-3.2.7-3.88-1.36-3.88-1.36-.52-1.33-1.28-1.68-1.28-1.68-1.04-.72.08-.7.08-.7 1.15.08 1.76 1.18 1.76 1.18 1.03 1.75 2.69 1.25 3.35.96.1-.75.4-1.25.73-1.54-2.55-.29-5.24-1.28-5.24-5.68 0-1.25.45-2.28 1.18-3.08-.12-.29-.51-1.46.11-3.04 0 0 .96-.31 3.15 1.18a10.9 10.9 0 0 1 5.73 0c2.19-1.49 3.15-1.18 3.15-1.18.62 1.58.23 2.75.11 3.04.74.8 1.18 1.83 1.18 3.08 0 4.41-2.69 5.38-5.25 5.67.41.36.78 1.06.78 2.14 0 1.55-.01 2.79-.01 3.17 0 .3.2.66.79.55A10.53 10.53 0 0 0 23.5 12C23.5 5.73 18.27.5 12 .5Z"
      />
    </svg>
  </div>
{/if}
