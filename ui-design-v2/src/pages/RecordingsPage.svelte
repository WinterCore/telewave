<script>
  import { fade, fly } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { RECORDINGS, TYPES, SORTS } from "../lib/channel.js";
  import RecordingRow from "../lib/components/RecordingRow.svelte";
  import PlayerBar from "../lib/components/PlayerBar.svelte";
  import EmptyState from "../lib/components/EmptyState.svelte";
  import { player } from "../lib/stores/player.svelte.js";

  let query = $state("");
  let type = $state("all");
  let sort = $state("newest");

  const visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const list = RECORDINGS.filter(
      (recording) =>
        (type === "all" || recording.type === type) &&
        (!q ||
          [recording.title, recording.author, recording.description].some(
            (field) => field.toLowerCase().includes(q),
          )),
    );
    const sorters = {
      newest: (a, b) => b.date.localeCompare(a.date),
      oldest: (a, b) => a.date.localeCompare(b.date),
      shortest: (a, b) => a.duration - b.duration,
      title: (a, b) => a.title.localeCompare(b.title),
    };
    return [...list].sort(sorters[sort]);
  });

  const countLabel = $derived(
    `${visible.length} ${visible.length === 1 ? "recording" : "recordings"}`,
  );

  function clearFilters() {
    query = "";
    type = "all";
  }
</script>

<section
  aria-labelledby="recordings-heading"
  class="pt-9 sm:pt-12 {player.recordingId ? 'pb-44 sm:pb-36' : ''}"
>
  <div class="flex items-end justify-between gap-4">
    <h2
      id="recordings-heading"
      class="font-display text-2xl font-semibold tracking-tight text-ink sm:text-[1.75rem]"
    >
      All recordings
    </h2>
    <p
      class="pb-1 font-mono text-xs uppercase tracking-[0.14em] text-muted"
      aria-live="polite"
    >
      {countLabel}
    </p>
  </div>

  <div
    class="mt-7 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between"
  >
    <label class="relative block w-full lg:max-w-xs">
      <span class="sr-only">Search recordings</span>
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.75"
        class="pointer-events-none absolute inset-y-0 start-3.5 my-auto size-4 text-muted"
        aria-hidden="true"
      >
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.8-3.8" stroke-linecap="round" />
      </svg>
      <input
        type="search"
        bind:value={query}
        placeholder="Search title, author, or description"
        class="h-10 w-full rounded-xl border border-line bg-surface ps-10 pe-3 text-sm text-ink outline-none transition placeholder:text-muted/60 focus:border-accent/60 focus:ring-2 focus:ring-accent/25"
      />
    </label>

    <div class="flex flex-wrap items-center gap-2">
      <div class="flex flex-wrap gap-1.5" role="group" aria-label="Filter by type">
        {#each TYPES as t (t.value)}
          <button
            type="button"
            aria-pressed={type === t.value}
            onclick={() => (type = t.value)}
            class="inline-flex h-9 cursor-pointer items-center rounded-full border px-3.5 text-[13px] font-medium transition focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent {type ===
            t.value
              ? "border-accent bg-accent text-accent-ink"
              : "border-muted/40 text-ink/75 hover:border-accent/60 hover:text-ink"}"
          >
            {t.label}
          </button>
        {/each}
      </div>

      <div class="relative ms-auto lg:ms-2">
        <label class="sr-only" for="sort-select">Sort recordings</label>
        <select
          id="sort-select"
          bind:value={sort}
          class="h-9 cursor-pointer appearance-none rounded-full border border-line bg-base ps-3.5 pe-8 text-[13px] font-medium text-muted outline-none transition hover:border-muted/60 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          {#each SORTS as s (s.value)}
            <option value={s.value}>{s.label}</option>
          {/each}
        </select>
        <svg
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          class="pointer-events-none absolute inset-y-0 end-3 my-auto size-3.5 text-muted"
          aria-hidden="true"
        >
          <path d="m4 6 4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </div>
    </div>
  </div>

  {#if visible.length}
    <div
      class="mt-6 divide-y divide-line overflow-hidden rounded-2xl border border-line bg-surface/85 shadow-2xl shadow-black/30 backdrop-blur-sm"
    >
      {#each visible as recording, index (recording.id)}
        <div
          animate:flip={{ duration: 260 }}
          in:fly={{ y: 10, duration: 220, delay: Math.min(index, 8) * 30 }}
        >
          <RecordingRow {recording} {index} />
        </div>
      {/each}
    </div>
  {:else}
    <div in:fade={{ duration: 220 }}>
      <EmptyState onClear={clearFilters} />
    </div>
  {/if}

  <PlayerBar />
</section>
