<script>
  let { onClear } = $props();
</script>

<div
  class="mt-6 rounded-2xl border border-dashed border-line bg-surface/60 px-6 py-16 text-center backdrop-blur-sm"
>
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.5"
    class="mx-auto size-8 text-muted/70"
    aria-hidden="true"
  >
    <circle cx="11" cy="11" r="7" />
    <path d="m20 20-3.8-3.8" stroke-linecap="round" />
    <path d="m8.8 8.8 4.4 4.4M13.2 8.8l-4.4 4.4" stroke-linecap="round" />
  </svg>
  <h3 class="mt-4 font-display text-lg font-semibold text-ink">
    Nothing matches
  </h3>
  <p class="mx-auto mt-1 max-w-xs text-sm leading-relaxed text-muted">
    No recordings fit your search and filters. Clear them to see everything
    again.
  </p>
  <button
    type="button"
    onclick={onClear}
    class="mt-6 inline-flex h-9 cursor-pointer items-center rounded-full border border-accent/50 px-4 text-sm font-medium text-accent transition hover:bg-accent/10 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
  >
    Clear filters
  </button>
</div>
