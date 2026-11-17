import { RootProvider } from 'fumadocs-ui/provider/next';
import { Banner } from 'fumadocs-ui/components/banner';
import type { Metadata } from 'next';
import Link from 'next/link';
import { Inter, JetBrains_Mono } from 'next/font/google';
import { appName, shipsternVersion } from '@/lib/shared';
import { siteUrl } from '@/lib/site-url';
import './global.css';

const inter = Inter({
  subsets: ['latin'],
  variable: '--font-inter',
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-jetbrains-mono',
});

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    template: `%s - ${appName}`,
    default: appName,
  },
  description:
    'Build program-aware, real-time Solana data pipelines in Rust with Shipstern: sources, parsers, handlers and prefilters.',
  icons: { icon: '/favicon.svg' },
};

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <html
      lang="en"
      className={`${inter.variable} ${jetbrainsMono.variable} font-sans`}
      suppressHydrationWarning
    >
      <body className="flex min-h-screen flex-col">
        <RootProvider theme={{ defaultTheme: 'dark' }}>
          <Banner id={`release-${shipsternVersion}`}>
            <span className="px-10 text-center text-sm">
              Shipstern {shipsternVersion} is out.{' '}
              <Link
                href={`/docs/releases/v${shipsternVersion.replaceAll('.', '-')}`}
                className="font-medium underline underline-offset-4"
              >
                Read the release notes
              </Link>
            </span>
          </Banner>
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
