<script>
  /**
   * Calm layered gradient wash behind both pages. Colors derive from the
   * channel theme, so rebranding recolors the background too.
   */
  let { class: cls = "" } = $props();
</script>

<div
  class="pointer-events-none fixed inset-0 -z-10 {cls}"
  style:background={`
    radial-gradient(52rem 36rem at 12% -8%, color-mix(in oklab, var(--brand-accent) 13%, transparent), transparent 60%),
    radial-gradient(44rem 30rem at 88% 10%, color-mix(in oklab, var(--brand-accent) 7%, transparent), transparent 60%),
    radial-gradient(64rem 44rem at 50% 118%, color-mix(in oklab, var(--brand-ink) 6%, transparent), transparent 65%)`}
  aria-hidden="true"
></div>
