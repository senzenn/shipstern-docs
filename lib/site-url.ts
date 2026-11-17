/** Public URL of the deployed site, for absolute links in metadata and the sitemap. */
export const siteUrl = (process.env.NEXT_PUBLIC_SITE_URL ?? 'http://localhost:3000').replace(/\/$/, '');
