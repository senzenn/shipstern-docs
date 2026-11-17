import { createGetUrl } from 'fumadocs-core/source';

export const appName = 'Shipstern Docs';
export const docsRoute = '/docs';
export const docsImageRoute = '/og/docs';
export const docsContentRoute = '/llms.mdx/docs';

/** The release the docs are written against. Bump it with every release page. */
export const shipsternVersion = '0.13.0';

/** The Shipstern framework repository (code, issues, releases). */
export const shipsternRepo = 'https://github.com/solana-rpc/shipstern';

/**
 * A new issue in the Shipstern repository with the page filled in. The docs repo is private,
 * so "Suggest an edit" opens an issue too, not an editor.
 */
export function getPageIssueUrl(page: { url: string }, title: string, kind: 'edit' | 'issue') {
  const query = new URLSearchParams(
    kind === 'edit'
      ? { title: `Docs edit: ${title}`, body: `Page: ${page.url}\n\nSuggested change:\n` }
      : { title: `Docs: ${title}`, body: `Page: ${page.url}\n\nProblem:\n` },
  );

  return `${shipsternRepo}/issues/new?${query}`;
}

const getContentUrl = createGetUrl(docsContentRoute);

export function getPageMarkdownUrl(page: { slugs: string[]; locale?: string }) {
  const segments = [...page.slugs, 'content.md'];

  return { segments, url: getContentUrl(segments, page.locale) };
}

const getImageUrl = createGetUrl(docsImageRoute);

export function getPageImageUrl(page: { slugs: string[]; locale?: string }) {
  const segments = [...page.slugs, 'image.png'];

  return { segments, url: getImageUrl(segments, page.locale) };
}
