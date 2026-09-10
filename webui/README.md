# Telewave web UI

Minimal SvelteKit application using Svelte 5, TypeScript, and Vite, scaffolded with
the [official Svelte CLI](https://svelte.dev/docs/cli/sv-create).

## Development

Install dependencies from the repository root with `npm install`. The root npm
workspace manages dependencies and the shared `package-lock.json`.

From the repository root:

```sh
npm run dev
npm run check
npm run build
npm run preview
```

The same commands also work from `webui/`. The development server runs at
<http://localhost:5173>.

## Structure

- `src/routes/`: SvelteKit pages and layouts.
- `src/lib/`: Shared components and utilities, imported through `$lib`.
- `static/`: Public static assets.

The initial page is a placeholder. The crawler and the existing `ui-design/`
prototypes are separate; backend integration and the application UI can be added
here.

The project uses the automatic SvelteKit adapter. Choose a deployment-specific
[adapter](https://svelte.dev/docs/kit/adapters) when a hosting target is selected.
