<script>
  import { fade } from "svelte/transition";
  import ChannelHeader from "./lib/components/ChannelHeader.svelte";
  import BrandFooter from "./lib/components/BrandFooter.svelte";
  import RecordingsPage from "./pages/RecordingsPage.svelte";
  import LivePage from "./pages/LivePage.svelte";
  import GradientBackground from "./lib/components/effects/GradientBackground.svelte";
  import { CHANNEL } from "./lib/channel.js";
  import { router } from "./lib/stores/router.svelte.js";

  const theme = CHANNEL.theme;
</script>

<svelte:head>
  <title>
    {router.page === "live" ? "البث المباشر" : "التسجيلات"} — {CHANNEL.name}
  </title>
  <meta name="description" content={CHANNEL.description} />
</svelte:head>

<div
  class="relative min-h-dvh font-sans text-ink antialiased"
  style:--brand-background={theme.background}
  style:--brand-surface={theme.surface}
  style:--brand-ink={theme.ink}
  style:--brand-muted={theme.muted}
  style:--brand-line={theme.line}
  style:--brand-accent={theme.accent}
  style:--brand-accent-ink={theme.accentInk}
  style:--brand-live={theme.live}
  style:--brand-heading-font={theme.headingFont}
  style:--brand-body-font={theme.bodyFont}
  style:--brand-mono-font={theme.monoFont}
>
  <GradientBackground />
  <!-- Vignette that fades the gradient into the page edges. -->
  <div
    class="pointer-events-none fixed inset-0 -z-10"
    style:background="radial-gradient(120% 90% at 50% 40%, transparent 55%, var(--brand-background) 130%)"
    aria-hidden="true"
  ></div>
  <a
    href="#main"
    class="sr-only focus:not-sr-only focus:fixed focus:start-4 focus:top-4 focus:z-50 focus:rounded-full focus:bg-accent focus:px-4 focus:py-2 focus:text-sm focus:font-medium focus:text-accent-ink"
  >
    انتقل إلى المحتوى
  </a>

  <div class="mx-auto w-full max-w-5xl px-5 sm:px-8">
    <ChannelHeader page={router.page} />
    <main id="main" tabindex="-1" class="focus:outline-none">
      {#key router.page}
        <div in:fade={{ duration: 200 }}>
          {#if router.page === "live"}
            <LivePage />
          {:else}
            <RecordingsPage />
          {/if}
        </div>
      {/key}
    </main>
    <BrandFooter />
  </div>
</div>
