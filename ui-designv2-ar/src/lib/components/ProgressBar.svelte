<script>
  import { formatTime } from "../format.js";

  /**
   * Shared progress track. `seekable` adds pointer dragging, keyboard
   * control, and a visible thumb; otherwise it renders read-only.
   */
  let { value, max, seekable = false, onSeek, label = "تقدّم التشغيل", class: cls = "" } = $props();

  let track;
  let dragging = $state(false);

  const pct = $derived(max > 0 ? Math.min(100, (value / max) * 100) : 0);

  function positionAt(event) {
    const rect = track.getBoundingClientRect();
    const ratio = (rect.right - event.clientX) / rect.width;
    return Math.min(1, Math.max(0, ratio)) * max;
  }

  function onPointerDown(event) {
    if (!seekable || event.button !== 0) return;
    dragging = true;
    track.setPointerCapture(event.pointerId);
    onSeek(positionAt(event));
  }

  function onPointerMove(event) {
    if (dragging) onSeek(positionAt(event));
  }

  function onPointerEnd() {
    dragging = false;
  }

  function onKeydown(event) {
    if (!seekable) return;
    const steps = { ArrowLeft: 10, ArrowRight: -10 };
    if (event.key in steps) {
      onSeek(Math.min(max, Math.max(0, value + steps[event.key])));
      event.preventDefault();
    } else if (event.key === "Home") {
      onSeek(0);
      event.preventDefault();
    } else if (event.key === "End") {
      onSeek(max);
      event.preventDefault();
    }
  }
</script>

<!-- Focusable only when it acts as role="slider" (seekable). -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  bind:this={track}
  role={seekable ? "slider" : "progressbar"}
  tabindex={seekable ? 0 : undefined}
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={Math.round(max)}
  aria-valuenow={Math.round(value)}
  aria-valuetext="{formatTime(value)} من {formatTime(max)}"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerEnd}
  onpointercancel={onPointerEnd}
  onkeydown={onKeydown}
  class="group relative flex h-5 items-center {seekable ? 'cursor-pointer touch-none focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-accent' : 'cursor-default'} {cls}"
>
  <div class="h-1 w-full overflow-hidden rounded-full bg-ink/10">
    <div class="h-full rounded-full bg-accent" style:width="{pct}%"></div>
  </div>
  {#if seekable}
    <div
      class="pointer-events-none absolute top-1/2 size-3 -translate-y-1/2 rounded-full bg-accent opacity-0 shadow-lg transition-opacity group-focus-visible:opacity-100 group-hover:opacity-100"
      class:opacity-100={dragging}
      style:inset-inline-start="calc({pct}% - 6px)"
      aria-hidden="true"
    ></div>
  {/if}
</div>
