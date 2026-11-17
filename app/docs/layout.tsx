import { source } from '@/lib/source';
import { DocsLayout } from 'fumadocs-ui/layouts/notebook';
import { baseOptions } from '@/lib/layout.shared';

/**
 * Notebook layout with the root folders as navbar tabs: logo, tabs and actions
 * across the top, section sidebar on the left, table of contents on the right.
 */
export default function Layout({ children }: LayoutProps<'/docs'>) {
  const { nav, ...base } = baseOptions();

  return (
    <DocsLayout
      {...base}
      tree={source.getPageTree()}
      tabMode="navbar"
      nav={{ ...nav, mode: 'top' }}
    >
      {children}
    </DocsLayout>
  );
}
