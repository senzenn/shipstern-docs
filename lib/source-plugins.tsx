import type { LoaderPlugin } from 'fumadocs-core/source';

/**
 * Show a page's `sidebarTitle` in the sidebar instead of its `title`.
 *
 * Page titles are unique site-wide and carry their section, for example
 * "Yellowstone gRPC Source", so they read well as the browser title and in
 * search. The sidebar already gives that context, so it uses the short label.
 */
export function sidebarTitlePlugin(): LoaderPlugin {
  return {
    name: 'shipstern:sidebar-title',
    enforce: 'pre',
    transformPageTree: {
      file(node, filePath) {
        if (!filePath) return node;

        const file = this.storage.read(filePath);
        if (file?.format !== 'page') return node;

        const { sidebarTitle } = file.data as { sidebarTitle?: unknown };
        if (typeof sidebarTitle === 'string') node.name = sidebarTitle;

        return node;
      },
    },
  };
}

const statusStyles: Record<string, string> = {
  new: 'bg-fd-primary/15 text-fd-primary',
  beta: 'bg-fd-warning/15 text-fd-warning',
  experimental: 'bg-fd-warning/15 text-fd-warning',
  deprecated: 'bg-fd-error/15 text-fd-error',
};

/** Sidebar badge for the `status` frontmatter field. */
export function renderStatusBadge(status: string) {
  return (
    <span
      className={`ms-auto shrink-0 whitespace-nowrap rounded-full px-1.5 py-px text-[0.625rem] font-semibold uppercase tracking-wide ${
        statusStyles[status] ?? 'bg-fd-muted text-fd-muted-foreground'
      }`}
    >
      {status}
    </span>
  );
}
