# Contributing to the Shipstern docs

This repository holds the documentation site only. Changes to Shipstern itself go to [solana-rpc/shipstern](https://github.com/solana-rpc/shipstern).

## Rules

- [AGENTS.md](./AGENTS.md) is the writing guide: accuracy, frontmatter, page patterns, voice, diagrams, code blocks and links.
- Check every API name, config key, default and code sample against the Shipstern release in `lib/shared.ts` (`shipsternVersion`). Read the source at that tag, not `main`.
- Rust code lives in `examples/`, never in MDX. CI compiles it against the documented release.
- Do not document unreleased changes on a released version's pages.

## Set up

Requirements:

- Node.js 22 or newer.
- For the Rust examples: Rust 1.97.1 or newer and the system packages in [the installation page](./content/docs/get-started/installation.mdx), which has commands for macOS, Debian/Ubuntu, RHEL/Fedora and Arch. `examples/kafka-sink` also needs CMake and the zlib headers.

```bash
npm ci
npm run dev        # http://localhost:3000
```

## Project layout

```text
app/
  docs/                 Docs layout (navbar tabs) and page renderer
  api/search/           Search index (Orama)
  llms.txt/, llms-full.txt/, llms.mdx/   Machine-readable docs
  og/                   Open Graph images
  theme.css             Colour tokens (the only place colours are defined)
  global.css            Tailwind, fonts, prose tweaks
components/
  mdx.tsx               Components available in MDX
  docs-card.tsx         Cards and card grids for landing pages
  code-block.tsx        Code blocks with the "Copy code" button
  mermaid.tsx           Mermaid diagrams: theme, fit to the column, wide flowcharts turn top-to-bottom
  update.tsx            Changelog entries (<Update>) and tags (<Tag>)
  logo.tsx              Logo and wordmark (public/logo.png)
content/docs/
  get-started/          Tab: Get Started
  (docs)/               Tab: Docs (runtime, sources, parsers, filtering, outputs, observability, testing)
  guides/               Tab: Guides
  reference/            Tab: Reference
  releases/             Tab: Releases (changelog, release notes, migration guides)
lib/
  shared.ts             Site name, Shipstern version, repository links
  source.ts             Content loader and frontmatter schema
examples/               Cargo workspace with every Rust program the docs show
scripts/
  lint-content.mjs      Content linter
templates/
  release-notes.mdx     Template for a new release page
```

Each top-level folder in `content/docs` with `"root": true` in its `meta.json` becomes a navbar tab. Page order comes from the `pages` array in each `meta.json`.

## Add or change a page

1. Create `content/docs/<section>/<slug>.mdx` with the frontmatter from AGENTS.md.
2. Add the slug to the folder's `meta.json`.
3. Put any Rust code in `examples/` and embed it (next section).
4. Open the page on the dev server and check it at desktop and phone widths. Check every diagram.
5. Run the checks.

## Write Rust samples

Pages embed Rust from `examples/`:

```mdx
<include cwd lang="rust" meta='title="src/main.rs"'>./examples/quickstart/src/main.rs</include>
<include cwd lang="rust">./examples/snippets/src/handlers.rs#logger</include>
```

The second form embeds the `// #region logger` block of a file.

- Add a whole program as a new crate in the `examples/` workspace, or add a region to a file in `examples/snippets`.
- `stream-server`, `kafka-sink`, `pump-trade-events`, `advanced-parsers`, `stream-replay` and `all-sources` are separate workspaces. The `proto` and `program-events` features of the first five would otherwise change the parser output types of the other examples. `all-sources` builds the Agave ledger crates for the snapshot and Jetstreamer sources. `kafka-sink`, `stream-replay` and `all-sources` take Shipstern crates from git.
- Keep comments inside a region short. Readers see them on the page.
- Commit `Cargo.lock`. CI builds with `--locked`, because the lockfile is what readers reproduce.

## Run an example

1. Copy the sample config: `cp examples/Shipstern.example.toml examples/Shipstern.toml`.
2. Set `endpoint` and `x-token` in `examples/Shipstern.toml`. The file holds credentials, so do not commit it.
3. From `examples/`, run `RUST_LOG=info cargo run -p quickstart -- --config Shipstern.toml`. For a separate workspace, use `--manifest-path stream-server/Cargo.toml` in place of `-p`.

## Run the checks

| Command | What it does |
| --- | --- |
| `npm run lint:content` | Content linter: frontmatter, banned words, em dashes, callout count, internal links and anchors. |
| `npm run lint` | ESLint. |
| `npm run types:check` | Generate route types and run `tsc`. |
| `npm run build` | Production build. Fails on broken MDX, unknown components or type errors. |
| `npm run check` | All four commands above, in that order. |
| `npm run check:examples` | `cargo check` of every Rust example. |
| `npm run test:examples` | Offline tests: instruction context, Token-2022 extensions, generated parsers and stream replay. |

Run `npm run check` before every pull request. Run `check:examples` and `test:examples` when a change touches `examples/` or the Shipstern version.

Stop `npm run dev` before `npm run build`. Both use `.next`, and the build stops the dev server.

## Update to a new Shipstern release

For every release, patch or minor:

1. Copy `templates/release-notes.mdx` to `content/docs/releases/vX-Y-Z.mdx`.
2. Fill it from the release's `CHANGELOG.md` entry and GitHub release. Put the upgrade impact table first.
3. Add the slug to `content/docs/releases/meta.json`, after `index`.
4. Add an `<Update>` entry at the top of `content/docs/releases/index.mdx`.
5. Set `shipsternVersion` in `lib/shared.ts`. This updates the site banner.
6. Set the same version in `examples/Cargo.toml` and in the separate workspaces. Then run `npm run check:examples` and `npm run test:examples`.
7. If the release has breaking changes, add a migration guide under `content/docs/releases/migrations/`.

## Open a pull request

- Branch from the default branch.
- Use a short imperative title, for example "Add a Fumarole guide".
- In the description, list the pages you changed and the Shipstern files or tags you checked them against.
- Say which claims you could not test, for example a guide that needs a live database.

CI runs on every pull request:

- **Site:** content lint, ESLint, types and build.
- **Rust examples:** `cargo check --locked` of every workspace and the offline tests.
- **Weekly:** the examples again with fresh dependency resolution, the way a new project resolves them. This finds upstream breaks before readers do.

Maintainers can run the devnet fixture tests from the Actions tab (`workflow_dispatch` with `run_fixture_tests`).
