import { Analytics } from '@vercel/analytics/react';
import { RootProvider } from 'fumadocs-ui/provider/next';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import type { ReactNode } from 'react';
import { source } from '@/lib/source';
import { SidebarLogo } from '@/components/themed-logo';
import './globals.css';

export const metadata = {
  title: {
    template: '%s | axel docs',
    default: 'axel Documentation',
  },
  description: 'Documentation for axel - portable agents across LLMs, reproducible terminal workspaces',
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body className="flex min-h-screen flex-col">
        <RootProvider
          theme={{
            defaultTheme: 'dark',
            attribute: 'class',
          }}
        >
          <DocsLayout
            tree={source.pageTree}
            nav={{
              title: <SidebarLogo />,
            }}
          >
            {children}
          </DocsLayout>
        </RootProvider>
        <Analytics />
      </body>
    </html>
  );
}
