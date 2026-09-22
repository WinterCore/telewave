# Telewave UI design — v2

High-fidelity prototype of Telewave's two public channel pages
(**Recordings** and **Live stream**), built with Svelte 5 and Tailwind CSS 4.

Everything is simulated: playback, seeking, and the live broadcast run off
in-memory state and the wall clock. There is no audio and no backend.

## Run it

```sh
npm install
npm run dev
```

## Rebrand a channel

All branding lives in `src/lib/channel.js` — logo, colors, fonts, and the
recordings themselves. Colors and fonts are applied as `--brand-*` custom
properties on the app root (`src/App.svelte`); Tailwind utilities reference
them through `@theme inline` in `src/app.css`, so one edit re-skins both pages.

## Structure

- `src/lib/channel.js` — channel branding + sample recordings
- `src/lib/format.js` — time/date formatting helpers
- `src/lib/stores/` — router, mock player, mock broadcast (Svelte 5 runes)
- `src/lib/components/` — shared UI components
- `src/lib/components/effects/` — ambient gradient background and a Svelte
  port of reactbits.dev "Split Text", plus spotlight rows, staggered list
  entrances, and FLIP reordering
- `src/pages/` — Recordings and Live stream pages
- `src/assets/artwork/` — hand-drawn SVG artwork
