<script>
  import { tick } from "svelte";
  import { formatNumber, normalizeSearch } from "../format.js";

  let {
    id,
    label,
    searchLabel,
    options,
    icon,
    align = "start",
    value = $bindable(),
  } = $props();

  let open = $state(false);
  let query = $state("");
  let activeIndex = $state(0);
  let root;
  let trigger;
  let searchInput = $state();
  let listbox = $state();

  const selected = $derived(options.find((option) => option.value === value));
  const filtered = $derived(
    options.filter((option) =>
      normalizeSearch(option.label).includes(normalizeSearch(query)),
    ),
  );
  const hasSelection = $derived(value !== options[0]?.value);

  async function showOptions() {
    query = "";
    activeIndex = Math.max(
      0,
      options.findIndex((option) => option.value === value),
    );
    open = true;
    await tick();
    searchInput?.focus();
    revealActiveOption();
  }

  function closeOptions(restoreFocus = false) {
    open = false;
    if (restoreFocus) trigger?.focus();
  }

  function selectOption(option) {
    value = option.value;
    closeOptions(true);
  }

  function dismissOutside(event) {
    if (open && !root?.contains(event.target)) closeOptions();
  }

  function revealActiveOption() {
    listbox?.children[activeIndex]?.scrollIntoView({ block: "nearest" });
  }

  async function handleSearchKey(event) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (!filtered.length) return;
      const direction = event.key === "ArrowDown" ? 1 : -1;
      activeIndex = (activeIndex + direction + filtered.length) % filtered.length;
      await tick();
      revealActiveOption();
    } else if (event.key === "Enter") {
      event.preventDefault();
      if (filtered[activeIndex]) selectOption(filtered[activeIndex]);
    }
  }
</script>

<svelte:window
  onpointerdown={dismissOutside}
  onfocusin={dismissOutside}
  onkeydown={(event) => {
    if (open && event.key === "Escape") {
      event.preventDefault();
      closeOptions(true);
    }
  }}
/>

<div class="relative min-w-0" bind:this={root}>
  <button
    bind:this={trigger}
    type="button"
    aria-label="{label}: {selected?.label}"
    aria-expanded={open}
    aria-haspopup="listbox"
    aria-controls={open ? `${id}-options` : undefined}
    onclick={() => (open ? closeOptions() : showOptions())}
    onkeydown={(event) => {
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        showOptions();
      }
    }}
    class="flex h-12 w-full cursor-pointer items-center gap-2 rounded-xl border px-2.5 text-start transition min-[380px]:gap-2.5 min-[380px]:px-3 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent {open || hasSelection
      ? 'border-accent/50 bg-accent/[0.06] text-accent'
      : 'border-line bg-surface text-muted hover:border-muted/60 hover:text-ink'}"
  >
    <span class="hidden shrink-0 min-[380px]:block" aria-hidden="true">
      {@render icon()}
    </span>
    <span class="min-w-0 flex-1">
      <span class="block text-[10px] font-medium text-muted">
        {label}
      </span>
      <span
        title={selected?.label}
        class="block truncate text-[13px] font-medium {hasSelection ? 'text-accent' : 'text-ink'}"
      >
        {selected?.label}
      </span>
    </span>
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      class="size-3.5 shrink-0 transition-transform {open ? 'rotate-180' : ''}"
      aria-hidden="true"
    >
      <path d="m4 6 4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>

  {#if open}
    <div
      class="absolute top-full z-30 mt-2 w-72 max-w-[calc(100vw-2.5rem)] overflow-hidden rounded-2xl border border-line bg-surface shadow-2xl shadow-black/50 {align === 'end' ? 'end-0' : 'start-0'}"
    >
      <div class="border-b border-line p-2.5">
        <div class="relative">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.75"
            class="pointer-events-none absolute start-3 top-3 size-4 text-muted"
            aria-hidden="true"
          >
            <circle cx="11" cy="11" r="7" />
            <path d="m20 20-3.8-3.8" stroke-linecap="round" />
          </svg>
          <input
            bind:this={searchInput}
            bind:value={query}
            oninput={() => (activeIndex = 0)}
            onkeydown={handleSearchKey}
            role="combobox"
            aria-label={searchLabel}
            aria-expanded="true"
            aria-autocomplete="list"
            aria-controls="{id}-options"
            aria-activedescendant={filtered[activeIndex] ? `${id}-option-${activeIndex}` : undefined}
            autocomplete="off"
            spellcheck="false"
            placeholder={searchLabel}
            class="h-10 w-full rounded-lg border border-line bg-base ps-9 pe-3 text-sm text-ink outline-none placeholder:text-muted/70 focus:border-accent/50 focus:ring-2 focus:ring-accent/15"
          />
        </div>
      </div>

      <ul
        bind:this={listbox}
        id="{id}-options"
        role="listbox"
        aria-label={label}
        class="max-h-64 scroll-py-1.5 overflow-y-auto overscroll-contain p-1.5 [scrollbar-width:thin]"
      >
        {#each filtered as option, index (option.value)}
          <li
            id="{id}-option-{index}"
            role="option"
            aria-selected={value === option.value}
            tabindex="-1"
            onclick={() => selectOption(option)}
            onpointermove={() => (activeIndex = index)}
            onkeydown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                selectOption(option);
              }
            }}
            class="flex min-h-11 cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-[13px] transition-colors {activeIndex === index ? 'bg-ink/[0.06]' : ''} {value === option.value ? 'font-medium text-accent' : 'text-ink'}"
          >
            <span class="min-w-0 flex-1 break-words">{option.label}</span>
            <span class="font-sans text-[11px] tabular-nums text-muted">{formatNumber(option.count)}</span>
            <span class="size-4 shrink-0" aria-hidden="true">
              {#if value === option.value}
                <svg
                  viewBox="0 0 16 16"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.75"
                >
                  <path d="m3.5 8 3 3 6-6" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              {/if}
            </span>
          </li>
        {/each}
      </ul>
      {#if !filtered.length}
        <p class="px-4 pb-5 pt-3 text-center text-sm text-muted" role="status">
          لا توجد نتائج مطابقة. جرّب بحثًا آخر.
        </p>
      {/if}
    </div>
  {/if}
</div>
