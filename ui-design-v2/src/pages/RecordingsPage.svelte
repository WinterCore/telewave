<script>
  import { fade, fly } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { RECORDINGS, TYPES, SORTS } from "../lib/channel.js";
  import RecordingRow from "../lib/components/RecordingRow.svelte";
  import PlayerBar from "../lib/components/PlayerBar.svelte";
  import EmptyState from "../lib/components/EmptyState.svelte";
  import FilterSelect from "../lib/components/FilterSelect.svelte";
  import { player } from "../lib/stores/player.svelte.js";

  let query = $state("");
  let type = $state("all");
  let author = $state("");
  let sort = $state("newest");

  const categories = TYPES.map((category) => ({
    ...category,
    label: category.value === "all" ? "All categories" : category.label,
    count: RECORDINGS.filter(
      (recording) => category.value === "all" || recording.type === category.value,
    ).length,
  }));
  const authors = [
    { value: "", label: "All authors", count: RECORDINGS.length },
    ...[...new Set(RECORDINGS.map((recording) => recording.author))]
      .sort((a, b) => a.localeCompare(b))
      .map((name) => ({
        value: name,
        label: name,
        count: RECORDINGS.filter((recording) => recording.author === name).length,
      })),
  ];
  const hasFilters = $derived(
    query.trim() !== "" || type !== "all" || author !== "",
  );

  const visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const list = RECORDINGS.filter(
      (recording) =>
        (type === "all" || recording.type === type) &&
        (!author || recording.author === author) &&
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
    author = "";
  }
</script>

<section
  aria-labelledby="recordings-heading"
  class="pt-9 sm:pt-12 {player.recordingId ? 'pb-44 sm:pb-36' : ''}"
>
  <h2
    id="recordings-heading"
    class="font-display text-2xl font-semibold tracking-tight text-ink sm:text-[1.75rem]"
  >
    All recordings
  </h2>

  <div
    class="mt-6 grid grid-cols-2 gap-3 lg:grid-cols-[minmax(0,1fr)_11.5rem_11.5rem]"
  >
    <label class="relative col-span-2 block min-w-0 lg:col-span-1">
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
        class="h-12 w-full rounded-xl border border-line bg-surface ps-10 pe-3 text-sm text-ink outline-none transition placeholder:text-muted/60 focus:border-accent/60 focus:ring-2 focus:ring-accent/25"
      />
    </label>

    <FilterSelect
      id="category-filter"
      label="Category"
      searchLabel="Search categories"
      options={categories}
      bind:value={type}
    >
      {#snippet icon()}
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          class="size-4"
        >
          <rect x="3.5" y="3.5" width="6" height="6" rx="1.5" />
          <rect x="14.5" y="3.5" width="6" height="6" rx="1.5" />
          <rect x="3.5" y="14.5" width="6" height="6" rx="1.5" />
          <rect x="14.5" y="14.5" width="6" height="6" rx="1.5" />
        </svg>
      {/snippet}
    </FilterSelect>

    <FilterSelect
      id="author-filter"
      label="Author"
      searchLabel="Search authors"
      options={authors}
      align="end"
      bind:value={author}
    >
      {#snippet icon()}
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          class="size-4"
        >
          <circle cx="12" cy="8" r="3.5" />
          <path d="M5 20v-1.5a7 7 0 0 1 14 0V20" stroke-linecap="round" />
        </svg>
      {/snippet}
    </FilterSelect>
  </div>

  <div class="mt-4 flex flex-wrap items-center justify-between gap-x-3 gap-y-2">
    <div class="flex flex-wrap items-center gap-x-3 gap-y-2">
      <p
        class="font-mono text-[11px] uppercase tracking-[0.12em] text-muted"
        aria-live="polite"
      >
        {countLabel}
      </p>
      {#if hasFilters}
        <button
          type="button"
          onclick={clearFilters}
          class="inline-flex min-h-9 cursor-pointer items-center gap-1 rounded-lg px-1 text-xs font-medium text-accent transition hover:text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          <svg
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            class="size-3"
            aria-hidden="true"
          >
            <path d="m4 4 8 8M12 4l-8 8" stroke-linecap="round" />
          </svg>
          Clear filters
        </button>
      {/if}
    </div>

    <div class="relative ms-auto">
      <label class="sr-only" for="sort-select">Sort recordings</label>
      <select
        id="sort-select"
        bind:value={sort}
        class="h-9 cursor-pointer appearance-none rounded-lg border border-transparent bg-transparent ps-2 pe-7 text-xs font-medium text-muted outline-none transition hover:border-line hover:text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
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
        class="pointer-events-none absolute inset-y-0 end-2 my-auto size-3.5 text-muted"
        aria-hidden="true"
      >
        <path d="m4 6 4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </div>
  </div>

  {#if visible.length}
    <div
      class="mt-3 divide-y divide-line overflow-hidden rounded-2xl border border-line bg-surface/85 shadow-2xl shadow-black/30 backdrop-blur-sm"
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
