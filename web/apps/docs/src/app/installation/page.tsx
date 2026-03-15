import type { Metadata } from 'next';
import { CommandBlock } from '@/components/code';
import { DocsPage, DocsBody } from 'fumadocs-ui/page';

export const metadata: Metadata = {
  title: 'Installation',
  description: 'How to install axel CLI and its dependencies',
};

const toc = [
  { title: 'Quick Install', url: '#quick-install', depth: 2 },
  { title: 'Prerequisites', url: '#prerequisites', depth: 2 },
  { title: 'Install with Cargo', url: '#install-with-cargo', depth: 2 },
  { title: 'Install tmux', url: '#install-tmux', depth: 2 },
  { title: 'Verify Installation', url: '#verify-installation', depth: 2 },
  { title: 'Next Steps', url: '#next-steps', depth: 2 },
];

export default function InstallationPage() {
  return (
    <DocsPage toc={toc}>
      <DocsBody>
        <h1 className="text-4xl font-bold text-[var(--foreground)] mb-4">Installation</h1>
      <p className="text-[var(--muted)] mb-8 text-lg">
        axel can be installed via our install script or built from source with Cargo.
      </p>

      <section className="mb-12">
        <h2 id="quick-install" className="text-2xl font-bold text-[var(--foreground)] mb-4">Quick Install</h2>
        <p className="text-[var(--muted)] mb-4">The fastest way to install axel:</p>
        <CommandBlock>curl -sL https://install.axel.rs | bash</CommandBlock>
      </section>

      <section className="mb-12">
        <h2 id="prerequisites" className="text-2xl font-bold text-[var(--foreground)] mb-4">Prerequisites</h2>
        <p className="text-[var(--muted)] mb-4">Before installing axel, ensure you have the following:</p>
        <ul className="list-disc list-inside text-[var(--muted)] space-y-2 mb-6">
          <li>
            <a href="https://github.com/tmux/tmux" className="text-[var(--accent)] hover:underline">
              tmux
            </a>{' '}
            - Terminal multiplexer (version 3.0 or later recommended)
          </li>
          <li>
            <a href="https://claude.ai/code" className="text-[var(--accent)] hover:underline">
              Claude Code
            </a>
            ,{' '}
            <a href="https://openai.com/codex" className="text-[var(--accent)] hover:underline">
              Codex
            </a>
            , or another AI coding assistant (optional)
          </li>
        </ul>
      </section>

      <section className="mb-12">
        <h2 id="install-with-cargo" className="text-2xl font-bold text-[var(--foreground)] mb-4">Install with Cargo</h2>
        <p className="text-[var(--muted)] mb-4">
          If you have{' '}
          <a href="https://www.rust-lang.org/tools/install" className="text-[var(--accent)] hover:underline">
            Rust
          </a>{' '}
          installed, you can build from source:
        </p>
        <CommandBlock>cargo install axel-cli</CommandBlock>
        <p className="text-[var(--muted)] mt-4">
          This will download, compile, and install the latest version of axel to your Cargo bin directory.
        </p>
      </section>

      <section className="mb-12">
        <h2 id="install-tmux" className="text-2xl font-bold text-[var(--foreground)] mb-4">Install tmux</h2>
        <p className="text-[var(--muted)] mb-4">If you do not have tmux installed, use your system package manager:</p>

        <p className="text-xs text-[var(--muted)] mb-1">macOS (Homebrew)</p>
        <CommandBlock>brew install tmux</CommandBlock>

        <p className="text-xs text-[var(--muted)] mb-1 mt-4">Ubuntu/Debian</p>
        <CommandBlock>sudo apt install tmux</CommandBlock>

        <p className="text-xs text-[var(--muted)] mb-1 mt-4">Fedora</p>
        <CommandBlock>sudo dnf install tmux</CommandBlock>

        <p className="text-xs text-[var(--muted)] mb-1 mt-4">Arch Linux</p>
        <CommandBlock>sudo pacman -S tmux</CommandBlock>
      </section>

      <section className="mb-12">
        <h2 id="verify-installation" className="text-2xl font-bold text-[var(--foreground)] mb-4">Verify Installation</h2>
        <p className="text-[var(--muted)] mb-4">Confirm axel is installed correctly:</p>
        <CommandBlock>axel --version</CommandBlock>
        <p className="text-[var(--muted)] mt-4">You should see the version number printed to the console.</p>
      </section>

      <section>
        <h2 id="next-steps" className="text-2xl font-bold text-[var(--foreground)] mb-4">Next Steps</h2>
        <p className="text-[var(--muted)]">
          Now that axel is installed, head to the{' '}
          <a href="/quick-start" className="text-[var(--accent)] hover:underline">
            Quick Start
          </a>{' '}
          guide to create your first workspace.
        </p>
      </section>
      </DocsBody>
    </DocsPage>
  );
}
