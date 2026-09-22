<script>
  /** Icon-only round play/pause button; size and coloring come from `class`. */
  let { playing = false, label, onclick, class: cls = "" } = $props();
</script>

<button
  type="button"
  {onclick}
  aria-label={label}
  title={label}
  class="grid shrink-0 cursor-pointer place-items-center rounded-full transition active:scale-95 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent {cls}"
>
  {#if playing}
    <svg viewBox="0 0 24 24" fill="currentColor" class="size-4" aria-hidden="true">
      <path d="M7 5h3.4v14H7zM13.6 5H17v14h-3.6z" />
    </svg>
  {:else}
    <svg viewBox="0 0 24 24" fill="currentColor" class="size-4 translate-x-px" aria-hidden="true">
      <path d="M8 5.5v13l11-6.5z" />
    </svg>
  {/if}
</button>
