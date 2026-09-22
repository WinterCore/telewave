<script>
  /** Small equalizer mark next to the selected recording; frozen when paused. */
  let { paused = false } = $props();
</script>

<span class="flex h-3.5 shrink-0 items-end gap-[3px]" aria-hidden="true">
  <span class="eq-bar h-full w-[3px] rounded-full bg-accent" class:is-paused={paused}></span>
  <span class="eq-bar h-full w-[3px] rounded-full bg-accent" class:is-paused={paused}></span>
  <span class="eq-bar h-full w-[3px] rounded-full bg-accent" class:is-paused={paused}></span>
</span>
