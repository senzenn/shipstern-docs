import defaultMdxComponents from 'fumadocs-ui/mdx';
import { Callout } from 'fumadocs-ui/components/callout';
import { Steps, Step } from 'fumadocs-ui/components/steps';
import { Tabs, Tab } from 'fumadocs-ui/components/tabs';
import { Accordions, Accordion } from 'fumadocs-ui/components/accordion';
import type { MDXComponents } from 'mdx/types';
import type { ReactNode } from 'react';
import { DocsCodeBlock } from './code-block';
import { DocsCard, DocsCards } from './docs-card';
import { Mermaid } from './mermaid';
import { Tag, Update } from './update';

type CalloutProps = { title?: ReactNode; children: ReactNode };

/**
 * Callout components for MDX pages.
 * House rule: at most two callouts per page.
 */
const Note = (props: CalloutProps) => <Callout type="info" {...props} />;
const Info = (props: CalloutProps) => <Callout type="info" {...props} />;
const Tip = (props: CalloutProps) => <Callout type="idea" {...props} />;
const Warning = (props: CalloutProps) => <Callout type="warning" {...props} />;

export function getMDXComponents(components?: MDXComponents) {
  return {
    ...defaultMdxComponents,
    Card: DocsCard,
    Cards: DocsCards,
    pre: DocsCodeBlock,
    Steps,
    Step,
    Tabs,
    Tab,
    Accordions,
    Accordion,
    Mermaid,
    Update,
    Tag,
    Note,
    Info,
    Tip,
    Warning,
    ...components,
  } satisfies MDXComponents;
}

declare global {
  type MDXProvidedComponents = ReturnType<typeof getMDXComponents>;
}
