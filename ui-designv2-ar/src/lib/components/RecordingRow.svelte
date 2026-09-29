<script>
  import Artwork from "./Artwork.svelte";
  import { typeLabel } from "../channel.js";
  import EqBars from "./EqBars.svelte";
  import { formatDate, formatTime, formatNumber } from "../format.js";
  import { player, toggle } from "../stores/player.svelte.js";

  let { recording, index } = $props();

  const selected = $derived(player.recordingId === recording.id);
  const playing = $derived(selected && player.playing);
</script>

<button
  type="button"
  onclick={() => toggle(recording.id)}
  aria-pressed={playing}
  aria-label="{playing ? 'إيقاف مؤقت' : 'تشغيل'}: {recording.title}، {recording.author}، {typeLabel(recording.type)}، {formatTime(recording.duration)}"
  class="group relative grid w-full cursor-pointer grid-cols-[auto_minmax(0,1fr)] items-center gap-2.5 px-2.5 py-2.5 text-start transition-colors duration-200 sm:grid-cols-[1.75rem_auto_minmax(0,1fr)_auto_auto] sm:gap-4 sm:px-4 sm:py-3 {selected
    ? "bg-accent/[0.08] before:absolute before:inset-y-0 before:start-0 before:w-0.5 before:bg-accent"
    : "hover:bg-ink/5"} focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
  onpointermove={(event) => {
    // Spotlight hover: keep a radial highlight under the cursor.
    const rect = event.currentTarget.getBoundingClientRect();
    event.currentTarget.style.setProperty("--mx", `${event.clientX - rect.left}px`);
    event.currentTarget.style.setProperty("--my", `${event.clientY - rect.top}px`);
  }}
>
  <div
    class="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-300 group-hover:opacity-100"
    style:background="radial-gradient(220px circle at var(--mx, 50%) var(--my, 50%), color-mix(in oklab, var(--brand-accent) 9%, transparent), transparent 70%)"
    aria-hidden="true"
  ></div>

  <span
    class="hidden self-center font-sans text-xs tabular-nums text-muted sm:block"
    aria-hidden="true"
  >
    {formatNumber(index + 1, 2)}
  </span>

  <span class="relative block shrink-0">
    <Artwork
      name={recording.image}
      alt=""
      class="size-10 rounded-lg ring-1 ring-white/10 sm:size-12"
    />
    {#if selected}
      <span
        class="absolute -bottom-1.5 -end-1.5 flex items-center rounded-md bg-base/90 px-1 py-1 ring-1 ring-white/10 backdrop-blur-sm"
        aria-hidden="true"
      >
        <EqBars paused={!playing} />
      </span>
    {/if}
  </span>

  <span class="block min-w-0">
    <span
      class="block truncate text-[13.5px] font-medium leading-snug transition-colors sm:text-[15px] {selected
        ? 'text-accent'
        : 'text-ink group-hover:text-accent'}"
    >
      {recording.title}
    </span>
    <span class="block truncate text-xs text-muted sm:text-[13px]">
      {recording.author} · {typeLabel(recording.type)}
      <span class="tabular-nums sm:hidden"> · <bdi dir="ltr">{formatTime(recording.duration)}</bdi></span>
    </span>
  </span>

  <time
    datetime={recording.date}
    class="hidden self-center text-[13px] text-muted sm:block"
  >
    {formatDate(recording.date)}
  </time>

  <span
    class="hidden self-center font-sans text-[13px] tabular-nums text-muted sm:block"
  >
    <bdi dir="ltr">{formatTime(recording.duration)}</bdi>
  </span>
</button>
