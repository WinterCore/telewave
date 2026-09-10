# Telewave

Browse and search live-audio recordings posted in Telegram channels.

## Repository layout

```text
crawler/    Rust crawler, TDLib submodule, and local Telegram session data
webui/      SvelteKit web application with TypeScript
ui-design/  Standalone HTML and Tailwind design prototypes
```

The web application is managed with npm workspaces and a root lockfile. The Rust
application keeps its own Cargo manifest and lockfile in `crawler/`. `ui-design/`
retains its standalone npm setup.

## Web development

Install dependencies and start the web app from the repository root:

```sh
npm install
npm run dev
```

The development server runs at <http://localhost:5173>. See
[webui/README.md](webui/README.md) for application details.

## Crawler development

Follow [crawler/README.md](crawler/README.md) to build TDLib and configure Telegram
credentials, then run the crawler from the repository root:

```sh
npm run dev:crawler
```

You can also run `cargo run` directly from `crawler/`. The crawler reads `.env`
and stores Telegram session data relative to that directory.

## Commands

| Command | Purpose |
| --- | --- |
| `npm run dev` | Start the SvelteKit development server |
| `npm run check` | Check Svelte and TypeScript |
| `npm run build` | Build the web application |
| `npm run preview` | Preview the web production build |
| `npm run dev:crawler` | Run the Rust crawler |
| `npm run check:crawler` | Check the Rust crawler |
| `npm run build:crawler` | Build and link the Rust crawler |

The existing design prototypes are documented in
[ui-design/README.md](ui-design/README.md).
