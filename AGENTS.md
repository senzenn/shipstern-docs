# Writing guide for the Shipstern docs

Read this before adding or editing a page.

## Accuracy

- Every API name, config key, default and code sample must match the Shipstern source at the version in `lib/shared.ts` (`shipsternVersion`). Check `solana-rpc/shipstern` before you write it down.
- Do not copy code from the crate READMEs without checking it. Several are out of date (the custom-source guide, `crates/parser/README.md`, `crates/mock/README.md`).
- Prefer code taken from `examples/*` or the crate's own tests, since those compile in CI, and link to the example the page is based on.
- Do not document unreleased changes on a released version's pages.

## Frontmatter

Every page has:

```yaml
title: "Unique, specific title"           # browser title, H1, search
sidebarTitle: "Short label"               # optional, when title is long
description: "One sentence for cards and the page header."
llmsDescription: "Detailed, keyword-rich description for llms.txt."
status: new                               # optional: new, beta, experimental, deprecated
```

## Page patterns

- **Concept page:** what it is, a simple diagram, details, how it affects your code, related links.
- **Guide:** one sentence on what you build, the source link, prerequisites, `<Steps>` with the full program embedded once, a "what each part does" table, see also.
- **Reference:** tables with type, default and description for every item.
- **Release notes:** copy `templates/release-notes.mdx`. The upgrade impact table comes first.

## Voice

- Direct and precise. Use neutral technical prose and imperative instructions. Name the source, parser, handler or application instead of addressing the reader as "you" or "your".
- No em dashes. Use a colon, comma or full stop.
- Avoid: simply, just, easy, please, leverage, utilize, seamless, supercharge, unlock, "in order to".
- Headings are statements ("Configure the source"), H2 and H3 only.
- British spelling (behaviour, initialise).
- At most two callouts (`<Note>`, `<Tip>`, `<Warning>`, `<Info>`) per page.

## Diagrams

- Use ```mermaid fences. Keep them small: one idea, plain labels, five to eight nodes.
- `flowchart LR` for data flow, `sequenceDiagram` for request and response order.
- No emojis, colours or styling in the diagram source; `components/mermaid.tsx` applies the theme.

## Code blocks

- **Rust code lives in `examples/`, not in MDX.** Put it in a crate there (a whole program, or a `// #region name` block in `examples/snippets`), run `cargo check --workspace --all-targets` in `examples/`, and embed it:

  ```mdx
  <include cwd lang="rust" meta='title="src/main.rs"'>./examples/quickstart/src/main.rs</include>
  <include cwd lang="rust">./examples/snippets/src/handlers.rs#logger</include>
  ```

  CI compiles that workspace against the released crates, so an embedded sample cannot silently break. Inline Rust is acceptable only for signatures and one-line fragments.
- Always set the language, and a `title="path/to/file"` when the block is a file.
- Never show the same program twice on a page.
- Every block gets a "Copy code" button automatically (`components/code-block.tsx`).

## Page endings

End with `## See also`: a short list of `- [Title](/docs/...): why you would go there`. Card grids are for landing pages only.

## Checks

`npm run check` runs the content linter (em dashes, banned words, missing frontmatter, broken `/docs` links and `#anchors`, links to the docs repo instead of Shipstern), lint, typecheck and build. CI runs it on every push.

## Components

Available in every MDX file without imports: `Cards`/`Card`, `Steps`/`Step`, `Tabs`/`Tab`, `Accordions`/`Accordion`, `Note`, `Tip`, `Warning`, `Info`, `Callout`, `Update`, `Tag`. Import icons from `lucide-react` at the top of the file when a card needs one.

## Links

- Internal: `/docs/...` paths, with `#anchor` for a section. The anchor is the heading slug (lowercase, spaces to `-`, code formatting dropped); release entries use `#v0-11-0`.
- GitHub links go to the Shipstern repository (`https://github.com/solana-rpc/shipstern`), never to this docs repo. For a guide, link the upstream example it is based on (`/tree/main/examples/...`).
- Shipstern source: link to `main` for files that exist there, and to tags (`/releases/tag/vX.Y.Z`) for version-specific notes.

<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->
