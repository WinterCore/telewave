<script>
  let { class: cls = "" } = $props();
</script>

<span
  class="inline-flex items-center gap-1.5 rounded-full bg-live/15 px-2.5 py-1 font-mono text-[11px] font-medium uppercase tracking-[0.14em] text-live ring-1 ring-live/40 backdrop-blur-sm {cls}"
>
  <span class="relative flex size-1.5">
    <span
      class="live-ping absolute inline-flex size-full rounded-full bg-live"
      aria-hidden="true"></span>
    <span class="relative inline-flex size-1.5 rounded-full bg-live"></span>
  </span>
  Live
</span>
