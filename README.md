# Shipstern Docs

Developer documentation for [Shipstern](https://github.com/solana-rpc/shipstern), the Rust framework for program-aware, real-time Solana data pipelines.

The site is a Next.js app built with [Fumadocs](https://fumadocs.dev). It has top navigation tabs, a section sidebar, an "On this page" outline, a Copy page menu that also opens the page in an AI chat, dark mode by default, and a generated `llms.txt`.

Every Rust sample on the site lives in `examples/` and compiles in CI against the documented Shipstern release.

## Run locally

Requires Node.js 22 or newer.

```bash
npm ci
npm run dev        # http://localhost:3000
```

Run `npm run check` before pushing. It runs the content linter, ESLint, the type check and a production build.

## Contribute

- [CONTRIBUTING.md](./CONTRIBUTING.md): set up, project layout, Rust samples, checks, release updates and pull requests.
- [AGENTS.md](./AGENTS.md): the writing guide for pages.

## Deploy

Any Next.js host works (Vercel, Netlify, a Node server). Set `NEXT_PUBLIC_SITE_URL` to the public URL so Open Graph images resolve.
