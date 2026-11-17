'use client';

import { Check, Copy } from 'lucide-react';
import { useEffect, useState, type MouseEvent } from 'react';

/**
 * Copy button shown on every code block. Copies the text of the block it sits
 * in, and confirms with a check mark for two seconds.
 */
export function CopyCode() {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;

    const timer = setTimeout(() => setCopied(false), 2000);
    return () => clearTimeout(timer);
  }, [copied]);

  async function copy(event: MouseEvent<HTMLButtonElement>) {
    const pre = event.currentTarget.closest('figure')?.querySelector('pre');
    if (!pre) return;

    await navigator.clipboard.writeText(pre.textContent ?? '');
    setCopied(true);
  }

  return (
    <button
      type="button"
      onClick={copy}
      aria-label={copied ? 'Copied' : 'Copy code'}
      title={copied ? 'Copied' : 'Copy code'}
      className="inline-flex size-7 items-center justify-center rounded-md border border-fd-border bg-fd-background/80 text-fd-muted-foreground transition-colors hover:text-fd-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-fd-ring"
    >
      {copied ? <Check className="size-3.5 text-fd-primary" /> : <Copy className="size-3.5" />}
      <span className="sr-only" aria-live="polite">
        {copied ? 'Copied' : ''}
      </span>
    </button>
  );
}
