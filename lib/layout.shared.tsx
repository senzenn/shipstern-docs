import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { Logo } from '@/components/logo';
import { shipsternRepo } from './shared';

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: <Logo />,
      url: '/docs/get-started',
    },
    githubUrl: shipsternRepo,
    links: [
      {
        type: 'button',
        text: 'Quickstart',
        url: '/docs/get-started/quickstart',
        on: 'nav',
      },
    ],
  };
}
