# Telewave channel pages

Two standalone Tailwind CSS pages for one Telegram channel:

- `index.html`: audio files, search, a type filter, sorting, and a basic player.
- `live.html`: the current file’s picture, metadata, progress, and play/pause.

Open either HTML file directly. Compiled CSS is included; no setup is required. Google Fonts are optional, with system font fallbacks.

## Channel branding

Edit `channel.js` to change the channel’s name, Telegram handle, description, logo, colors, and heading font. Both pages share these settings. This is a page template, with no admin or customization screen.

Example recordings are in `app.js`. Playback and the live broadcast are visual simulations without audio or backend integration.

## Editing styles

Most styles are Tailwind utilities in the HTML and recording template. Shared controls are in `src/tailwind.css`.

```sh
cd ui-design
npm ci
npm run build
```

Use `npm run watch` while editing. The generated stylesheet is `styles.css`.
