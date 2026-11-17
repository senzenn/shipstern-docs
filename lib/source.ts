import { llms, loader } from 'fumadocs-core/source';
import { lucideIconsPlugin } from 'fumadocs-core/source/lucide-icons';
import { statusBadgesPlugin } from 'fumadocs-core/source/status-badges';
import { remarkMdxMermaid } from 'fumadocs-core/mdx-plugins/remark-mdx-mermaid';
import { metaSchema, pageSchema } from 'fumadocs-core/source/schema';
import { applyMdxPreset } from 'fumadocs-mdx/config';
import { defineDocs } from 'fumadocs-mdx/macro';
import { z } from 'zod';
import { docsRoute } from './shared';
import { renderStatusBadge, sidebarTitlePlugin } from './source-plugins';

/**
 * Frontmatter for every page. `title` and `description` come from Fumadocs;
 * the rest are fields of this site:
 *
 * - `sidebarTitle`: short label for the sidebar when `title` is long.
 * - `llmsDescription`: detailed description used in `llms.txt`.
 * - `status`: sidebar badge (`new`, `beta`, `experimental`, `deprecated`).
 */
const shipsternPageSchema = pageSchema.extend({
  sidebarTitle: z.string().optional(),
  llmsDescription: z.string().optional(),
  status: z.enum(['new', 'beta', 'experimental', 'deprecated']).optional(),
});

const docs = defineDocs({
  dir: 'content/docs',
  docs: {
    schema: shipsternPageSchema,
    mdxOptions: applyMdxPreset({
      remarkPlugins: (plugins) => [...plugins, remarkMdxMermaid],
      rehypeCodeOptions: {
        themes: { light: 'github-light', dark: 'github-dark-default' },
        langs: ['rust', 'toml', 'bash', 'shell', 'json', 'yaml', 'protobuf', 'diff', 'sql'],
      },
    }),
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
  meta: {
    schema: metaSchema,
  },
});

export const source = loader({
  baseUrl: docsRoute,
  source: docs.toFumadocsSource(),
  plugins: [
    lucideIconsPlugin(),
    sidebarTitlePlugin(),
    statusBadgesPlugin({ renderBadge: renderStatusBadge }),
  ],
});

export const docsLlms = llms(source, {
  renderPage: async (page) => `# ${page.data.title} (${page.url})

${page.data.llmsDescription ?? page.data.description ?? ''}

${await page.data.getText('processed')}`,
});
