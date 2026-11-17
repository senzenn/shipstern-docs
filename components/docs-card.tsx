import Link from 'fumadocs-core/link';
import type { CardProps } from 'fumadocs-ui/components/card';
import type { HTMLAttributes } from 'react';

export function DocsCards({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      {...props}
      className={`not-prose grid grid-cols-1 gap-x-8 sm:grid-cols-2 ${className ?? ''}`}
    />
  );
}

export function DocsCard({
  icon, title, description, children, href, external, className, ...props
}: CardProps) {
  const content = (
    <>
      {icon && (
        <span
          aria-hidden="true"
          className="mt-0.5 shrink-0 text-fd-muted-foreground transition-colors group-hover:text-fd-foreground [&_svg]:size-5 [&_svg]:stroke-[1.75]"
        >
          {icon}
        </span>
      )}
      <div className="min-w-0">
        <h3 className="text-sm font-semibold text-fd-foreground underline-offset-4 group-hover:underline">{title}</h3>
        <div className="mt-1 text-sm leading-relaxed text-fd-muted-foreground">
          {description}
          {children}
        </div>
      </div>
    </>
  );
  const classes = `group flex items-start gap-3 border-b border-fd-border py-5 ${className ?? ''}`;

  if (href) {
    return (
      <Link
        {...props}
        href={href}
        external={external}
        className={`${classes} focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-fd-primary`}
      >
        {content}
      </Link>
    );
  }

  return <div {...props} className={classes}>{content}</div>;
}
