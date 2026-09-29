<script>
  import { CHANNEL } from "../channel.js";
  import SplitText from "./effects/SplitText.svelte";

  let { page } = $props();

  const tabs = [
    { id: "recordings", href: "#/", label: "التسجيلات" },
    { id: "live", href: "#/live", label: "البث المباشر", live: true },
  ];
</script>

<header class="pt-8 sm:pt-12">
  <div class="flex flex-wrap items-end justify-between gap-x-6 gap-y-5">
    <div class="flex min-w-0 items-center gap-4 sm:gap-5">
      <img
        src={CHANNEL.logo}
        alt="شعار {CHANNEL.name}"
        class="size-14 shrink-0 rounded-2xl ring-1 ring-white/10 sm:size-16"
      />
      <div class="min-w-0">
        <p
          class="font-sans text-[11px] text-muted"
        >
          <bdi dir="ltr" lang="en" class="channel-handle">{CHANNEL.handle}</bdi>
        </p>
        <h1
          class="mt-1 font-display text-[1.7rem] font-semibold leading-normal text-ink sm:text-4xl"
        >
          <SplitText text={CHANNEL.name} />
        </h1>
      </div>
    </div>

    <nav
      class="flex gap-1 rounded-full border border-line bg-surface/70 p-1 backdrop-blur"
      aria-label="صفحات القناة"
    >
      {#each tabs as tab (tab.id)}
        <a
          href={tab.href}
          aria-current={page === tab.id ? "page" : undefined}
          class="relative inline-flex h-9 items-center gap-2 rounded-full px-4 text-sm font-medium transition focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent {page === tab.id
            ? 'bg-accent text-accent-ink'
            : 'text-muted hover:text-ink'}"
        >
          {#if tab.live}
            <span class="relative flex size-1.5" aria-hidden="true">
              <span
                class="live-ping absolute inline-flex size-full rounded-full bg-live"
              ></span>
              <span class="relative inline-flex size-1.5 rounded-full bg-live"
              ></span>
            </span>
          {/if}
          {tab.label}
        </a>
      {/each}
    </nav>
  </div>

  <p class="mt-5 max-w-lg text-[15px] leading-relaxed text-muted">
    {CHANNEL.description}
  </p>
</header>
