'use client';

import { CodeBlock, Pre } from 'fumadocs-ui/components/codeblock';
import type { ComponentProps } from 'react';
import { CopyCode } from './copy-code';

/**
 * Every fenced code block in the docs: Shiki highlighting from Fumadocs, with
 * a labelled "Copy code" button in the top-right corner (or in the title bar
 * when the block has a `title`).
 */
export function DocsCodeBlock(props: ComponentProps<'pre'> & { title?: string; icon?: string }) {
  return (
    <CodeBlock
      {...props}
      allowCopy={false}
      Actions={({ className }) => (
        <div className={className}>
          <CopyCode />
        </div>
      )}
    >
      <Pre>{props.children}</Pre>
    </CodeBlock>
  );
}
