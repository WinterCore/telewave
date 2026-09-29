<script>
  import { fly } from "svelte/transition";
  import Artwork from "./Artwork.svelte";
  import PlayButton from "./PlayButton.svelte";
  import ProgressBar from "./ProgressBar.svelte";
  import { formatTime } from "../format.js";
  import { RECORDINGS } from "../channel.js";
  import { player, toggle, seek } from "../stores/player.svelte.js";

  const current = $derived(
    RECORDINGS.find((recording) => recording.id === player.recordingId),
  );
</script>

{#if current}
  <div
    class="fixed inset-x-0 bottom-0 z-40 border-t border-line bg-surface/95 backdrop-blur"
    transition:fly={{ y: 96, duration: 260 }}
    role="group"
    aria-label="مشغّل التسجيلات"
  >
    <div class="mx-auto max-w-5xl px-4 sm:px-8">
      <div class="flex items-center gap-3 py-2.5 sm:gap-4 sm:py-3.5">
        <Artwork
          name={current.image}
          alt=""
          class="size-10 shrink-0 rounded-lg ring-1 ring-white/10 sm:size-12"
        />
        <div class="min-w-0 flex-1 sm:w-52 sm:flex-none lg:w-64">
          <p class="text-sm font-medium leading-snug text-ink">{current.title}</p>
          <p class="truncate text-xs text-muted">{current.author}</p>
        </div>
        <PlayButton
          playing={player.playing}
          label={player.playing ? "إيقاف مؤقت" : "تشغيل"}
          onclick={() => toggle(current.id)}
          class="size-10 bg-accent text-accent-ink hover:opacity-90 sm:size-11"
        />
        <div class="hidden min-w-0 flex-1 items-center gap-3 sm:flex">
          <span class="font-sans text-xs tabular-nums text-muted">
            <bdi dir="ltr">{formatTime(player.position)}</bdi>
          </span>
          <ProgressBar
            seekable
            value={player.position}
            max={current.duration}
            onSeek={seek}
            label={"موضع التشغيل: " + current.title}
            class="min-w-0 flex-1"
          />
          <span class="font-sans text-xs tabular-nums text-muted">
            <bdi dir="ltr">{formatTime(current.duration)}</bdi>
          </span>
        </div>
      </div>
      <div class="flex items-center gap-3 pb-3 sm:hidden">
        <span class="min-w-9 shrink-0 font-sans text-xs tabular-nums text-muted">
          <bdi dir="ltr">{formatTime(player.position)}</bdi>
        </span>
        <ProgressBar
          seekable
          value={player.position}
          max={current.duration}
          onSeek={seek}
          label={"موضع التشغيل: " + current.title}
          class="min-w-0 flex-1"
        />
        <span
          class="min-w-9 shrink-0 text-end font-sans text-xs tabular-nums text-muted"
        >
          <bdi dir="ltr">{formatTime(current.duration)}</bdi>
        </span>
      </div>
    </div>
  </div>
{/if}
