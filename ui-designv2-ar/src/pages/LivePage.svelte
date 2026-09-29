<script>
  import { fade } from "svelte/transition";
  import Artwork from "../lib/components/Artwork.svelte";
  import LiveBadge from "../lib/components/LiveBadge.svelte";
  import ProgressBar from "../lib/components/ProgressBar.svelte";
  import SplitText from "../lib/components/effects/SplitText.svelte";
  import { typeLabel } from "../lib/channel.js";
  import { formatApprox, formatTime } from "../lib/format.js";
  import { broadcast, onAir, toggleListen } from "../lib/stores/broadcast.svelte.js";

  const live = $derived(onAir());
</script>

<section
  class="mx-auto flex max-w-md flex-col items-center overflow-x-clip pb-6 pt-10 text-center sm:pt-14"
  aria-labelledby="live-heading"
>
  <div class="relative w-full max-w-72 sm:max-w-80">
    <!-- Ambient amber glow behind the artwork. -->
    <div
      class="absolute -inset-8 -z-10 rounded-full bg-accent/15 blur-3xl"
      aria-hidden="true"
    ></div>
    {#key live.recording.id}
      <div in:fade={{ duration: 450 }}>
        <Artwork
          name={live.recording.image}
          alt={"غلاف تسجيل " + live.recording.title}
          class="aspect-square w-full rounded-2xl shadow-2xl shadow-black/50 ring-1 ring-white/10"
        />
      </div>
    {/key}
    <LiveBadge class="absolute start-3 top-3" />
  </div>

  <p
    class="mt-8 font-sans text-[11px] text-muted"
  >
    {typeLabel(live.recording.type)} · {formatApprox(live.recording.duration)}
  </p>
  <h2
    id="live-heading"
    class="mt-3 font-display text-2xl font-semibold leading-snug text-ink sm:text-[1.75rem]"
  >
    <SplitText text={live.recording.title} wordDelay={55} />
  </h2>
  <p class="mt-1.5 text-sm text-muted">{live.recording.author}</p>
  <p class="mt-4 max-w-sm text-sm leading-relaxed text-muted/90">
    {live.recording.description}
  </p>

  <div class="mt-9 w-full">
    <ProgressBar
      value={live.position}
      max={live.recording.duration}
      label="تقدّم البث المباشر"
    />
    <div class="mt-1 flex justify-between font-sans text-xs tabular-nums text-muted">
      <span><bdi dir="ltr">{formatTime(live.position)}</bdi></span>
      <span><bdi dir="ltr">{formatTime(live.recording.duration)}</bdi></span>
    </div>
  </div>

  <button
    type="button"
    onclick={toggleListen}
    aria-pressed={broadcast.listening}
    class="mt-7 inline-flex h-12 min-w-60 cursor-pointer items-center justify-center gap-2.5 rounded-full bg-accent px-8 text-[15px] font-semibold text-accent-ink shadow-lg shadow-accent/20 transition hover:opacity-90 active:scale-[0.98] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
  >
    {#if broadcast.listening}
      <svg viewBox="0 0 24 24" fill="currentColor" class="size-4.5" aria-hidden="true">
        <path d="M7 5h3.4v14H7zM13.6 5H17v14h-3.6z" />
      </svg>
      إيقاف مؤقت
    {:else}
      <svg
        viewBox="0 0 24 24"
        fill="currentColor"
        class="size-4.5 translate-x-px"
        aria-hidden="true"
      >
        <path d="M8 5.5v13l11-6.5z" />
      </svg>
      استمع الآن
    {/if}
  </button>
  <p class="mt-4 font-sans text-[11px] text-muted/70">
    {#if broadcast.listening}
      أنت تستمع إلى البث المباشر
    {:else}
      يستمر البث حتى عند إيقاف الاستماع مؤقتًا
    {/if}
  </p>
</section>
