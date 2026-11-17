import Link from 'next/link';
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { baseOptions } from '@/lib/layout.shared';

const suggestions = [
  { href: '/docs/get-started', label: 'Welcome' },
  { href: '/docs/get-started/quickstart', label: 'Quickstart' },
  { href: '/docs/reference/cheat-sheet', label: 'Cheat sheet' },
  { href: '/docs/releases', label: 'Changelog' },
];

export default function NotFound() {
  return (
    <HomeLayout {...baseOptions()}>
      <main className="mx-auto flex w-full max-w-xl flex-1 flex-col justify-center gap-4 px-6 py-24">
        <p className="font-mono text-sm text-fd-muted-foreground">404</p>
        <h1 className="text-2xl font-semibold">This page does not exist</h1>
        <p className="text-fd-muted-foreground">
          The link may be out of date. Use search (<kbd className="font-mono">Ctrl K</kbd>) or start
          from one of these pages:
        </p>
        <ul className="flex flex-wrap gap-2">
          {suggestions.map((s) => (
            <li key={s.href}>
              <Link
                href={s.href}
                className="inline-block rounded-md border border-fd-border px-3 py-1.5 text-sm hover:bg-fd-accent"
              >
                {s.label}
              </Link>
            </li>
          ))}
        </ul>
      </main>
    </HomeLayout>
  );
}
