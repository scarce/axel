import type { Metadata } from "next";
import { Analytics } from "@vercel/analytics/react";
import { Header } from "@/components/header";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "Axel - Accelerated Task Manager for Mac",
    template: "%s | Axel",
  },
  description:
    "Queue tasks, dispatch to Claude/Codex/OpenCode, manage permissions from one inbox. Native macOS app.",
  openGraph: {
    title: "Axel - Accelerated Task Manager for Mac",
    description:
      "Queue tasks, dispatch to Claude/Codex/OpenCode, manage permissions from one inbox. Native macOS app.",
    url: "https://axel.build",
    siteName: "Axel",
    images: [
      {
        url: "/og.png",
        width: 1200,
        height: 630,
        alt: "Axel - Accelerated Task Manager for Mac",
      },
    ],
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Axel - Accelerated Task Manager for Mac",
    description:
      "Queue tasks, dispatch to Claude/Codex/OpenCode, manage permissions from one inbox. Native macOS app.",
    images: ["/og.png"],
  },
};

function Footer() {
  return (
    <footer className="relative z-10 border-t border-gray-100 bg-white">
      <div className="mx-auto max-w-5xl px-6 py-8">
        <div className="flex flex-col items-center justify-between gap-4 sm:flex-row">
          <p className="text-sm text-gray-400">&copy; 2025 Txtx, Inc.</p>
          <div className="flex items-center gap-6">
            <a
              href="https://github.com/lgalabru/axel"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-gray-400 transition-colors hover:text-gray-600"
            >
              GitHub
            </a>
            <a
              href="https://docs.axel.build"
              className="text-sm text-gray-400 transition-colors hover:text-gray-600"
            >
              Docs
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <meta name="theme-color" content="#050510" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          rel="preconnect"
          href="https://fonts.gstatic.com"
          crossOrigin="anonymous"
        />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=Space+Grotesk:wght@700&display=swap"
          rel="stylesheet"
        />
      </head>
      <body className="min-h-screen flex flex-col bg-[#050510]">
        <Header />
        <main className="flex-1">{children}</main>
        <Footer />
        <Analytics />
      </body>
    </html>
  );
}
