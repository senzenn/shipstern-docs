import type { ReactNode } from 'react';

const tagStyles: Record<string, string> = {
  Breaking: 'border-fd-error/40 bg-fd-error/10 text-fd-error',
  Security: 'border-fd-error/40 bg-fd-error/10 text-fd-error',
  'Behaviour change': 'border-fd-warning/40 bg-fd-warning/10 text-fd-warning',
  Deprecated: 'border-fd-warning/40 bg-fd-warning/10 text-fd-warning',
  Added: 'border-fd-success/40 bg-fd-success/10 text-fd-success',
  Fixed: 'border-fd-info/40 bg-fd-info/10 text-fd-info',
  Patch: 'border-fd-primary/40 bg-fd-primary/10 text-fd-primary',
  Minor: 'border-fd-primary/40 bg-fd-primary/10 text-fd-primary',
};

/** Small coloured label for a release or a change type. */
export function Tag({ children }: { children: string }) {
  return (
    <span
      className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium ${
        tagStyles[children] ?? 'border-fd-border bg-fd-muted text-fd-muted-foreground'
      }`}
    >
      {children}
    </span>
  );
}

/**
 * One release on the changelog timeline: the date and version on the left, the
 * notes on the right and a rule between releases.
 *
 * ```mdx
 * <Update label="2026-09-18" version="v0.11.0" tags={["Minor", "Breaking"]} href="/docs/releases/v0-11-0">
 *   ...notes...
 * </Update>
 * ```
 */
export function Update({
  label,
  version,
  tags = [],
  href,
  children,
}: {
  /** Release date, `YYYY-MM-DD`. */
  label: string;
  /** Version string, for example `v0.11.0`. */
  version: string;
  tags?: string[];
  /** Link to the full release notes page. */
  href?: string;
  children: ReactNode;
}) {
  const anchor = version.replaceAll('.', '-');

  return (
    <section
      id={anchor}
      className="relative flex scroll-mt-28 flex-col gap-4 border-b border-fd-border py-10 first:pt-2 last:border-b-0 md:flex-row md:gap-10"
    >
      <div className="not-prose md:sticky md:top-28 md:w-40 md:shrink-0 md:self-start">
        <a href={`#${anchor}`} className="text-lg font-semibold text-fd-foreground hover:underline">
          {version}
        </a>
        <p className="mt-1 font-mono text-xs text-fd-muted-foreground">{label}</p>
        {tags.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-1.5">
            {tags.map((tag) => (
              <Tag key={tag}>{tag}</Tag>
            ))}
          </div>
        )}
      </div>
      <div className="min-w-0 flex-1 [&>*:first-child]:mt-0">
        {children}
        {href && (
          <p>
            <a href={href}>Full release notes for {version}</a>
          </p>
        )}
      </div>
    </section>
  );
}
