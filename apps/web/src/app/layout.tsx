import type { Metadata } from 'next';
import { Geist, Geist_Mono } from 'next/font/google';
import './globals.css';

const geistSans = Geist({
  variable: '--font-geist-sans',
  subsets: ['latin'],
});

const geistMono = Geist_Mono({
  variable: '--font-geist-mono',
  subsets: ['latin'],
});

export const metadata: Metadata = {
  title: {
    default: 'MCP Daddy',
    template: '%s | MCP Daddy',
  },
  description: 'A local-first MCP proxy that manages upstream servers and per-client exposure.',
  openGraph: {
    title: 'MCP Daddy',
    description: 'A local-first MCP proxy that manages upstream servers and per-client exposure.',
    type: 'website',
    images: ['/og.svg'],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'MCP Daddy',
    description: 'A local-first MCP proxy that manages upstream servers and per-client exposure.',
    images: ['/og.svg'],
  },
  icons: {
    icon: [{ url: '/favicon.svg', type: 'image/svg+xml' }],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} min-h-dvh font-sans antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
