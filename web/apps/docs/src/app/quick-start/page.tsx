import type { Metadata } from 'next';
import { CommandBlock, YamlBlock } from '@/components/code';
import { DocsPage, DocsBody } from 'fumadocs-ui/page';

export const metadata: Metadata = {
  title: 'Quick Start',
  description: 'Get started with axel CLI in minutes',
};

const toc = [
  { title: 'Part 1: Skills', url: '#part-1-skills', depth: 2 },
  { title: 'Part 2: Grid Layouts', url: '#part-2-grid-layouts', depth: 2 },
  { title: 'Next Steps', url: '#next-steps', depth: 2 },
];

function Step({ number, title, children }: { number: number; title: string; children: React.ReactNode }) {
  return (
    <div className="mb-10">
      <div className="flex items-center gap-4 mb-4">
        <div className="w-8 h-8 rounded-full bg-[var(--accent-dark)] text-[var(--accent)] flex items-center justify-center text-sm font-bold border border-[var(--accent)]/20">
          {number}
        </div>
        <h3 className="text-xl font-semibold text-[var(--foreground)]">{title}</h3>
      </div>
      <div className="ml-12">{children}</div>
    </div>
  );
}

export default function QuickStartPage() {
  return (
    <DocsPage toc={toc}>
      <DocsBody>
        <h1 className="text-4xl font-bold text-[var(--foreground)] mb-4">Quick Start</h1>
      <p className="text-[var(--muted)] mb-12 text-lg">Get up and running with axel in just a few minutes.</p>

      <section className="mb-16">
        <h2 id="part-1-skills" className="text-2xl font-bold text-[var(--foreground)] mb-6">Part 1: Skills</h2>

        <Step number={1} title="Import Your Skills">
          <p className="text-[var(--muted)] mb-4">
            Start by importing your existing skill files into axel&apos;s global directory:
          </p>
          <CommandBlock>axel skill import ./.claude/skills/web-developer</CommandBlock>
          <p className="text-[var(--muted)] mt-4 mb-4">
            You can also import an entire directory of skills:
          </p>
          <CommandBlock>axel skill import ./skills/</CommandBlock>
          <p className="text-[var(--muted)] mt-4 mb-4">
            Verify your imported skills:
          </p>
          <CommandBlock>axel skill ls</CommandBlock>
          <p className="text-[var(--muted)] mt-4">
            Skills are stored in{' '}
            <code className="text-[var(--accent)] font-medium">~/.config/axel/skills</code>,
            making them available across all your workspaces.
          </p>
          <div className="mt-4 rounded-lg border border-[var(--border)] bg-[var(--card)]/50 p-3">
            <p className="text-[var(--muted)] text-sm">
              <strong className="text-[var(--foreground)]">Experimental:</strong>{' '}
              <code className="text-[var(--accent)] text-xs">axel bootstrap</code> can auto-discover skills across your machine,
              but we recommend manual import for more control.
            </p>
          </div>
        </Step>

        <Step number={2} title="Initialize a Workspace">
          <p className="text-[var(--muted)] mb-4">
            In an existing repository, initialize an axel workspace:
          </p>
          <CommandBlock>axel init</CommandBlock>
          <p className="text-[var(--muted)] mt-4">
            This creates a minimal{' '}
            <code className="text-[var(--accent)] font-medium">AXEL.md</code> in your project:
          </p>
          <YamlBlock filename="AXEL.md">
{`workspace: my-project

layouts:
  panes:
    - type: claude
      skills:
        - "*"

  grids:
    default:
      type: tmux
      claude:
        col: 0
        row: 0`}
          </YamlBlock>
        </Step>

        <Step number={3} title="Launch Claude">
          <p className="text-[var(--muted)] mb-4">
            Launch the Claude pane to see your skills in action:
          </p>
          <CommandBlock>axel claude</CommandBlock>
          <p className="text-[var(--muted)] mt-4 mb-4">
            You&apos;ll see output like this:
          </p>
          <div className="bg-[var(--card)] rounded-lg border border-[var(--border)] p-4 font-mono text-sm">
            <div className="text-[var(--muted)]">$ axel claude</div>
            <div className="text-emerald-600 dark:text-emerald-400 mt-2">Installing skills...</div>
            <div className="text-[var(--muted)] ml-2">→ .claude/skills/web-developer (symlink)</div>
            <div className="text-[var(--muted)] ml-2">→ .claude/skills/rust-developer (symlink)</div>
            <div className="text-emerald-600 dark:text-emerald-400 mt-2">Launching claude...</div>
          </div>
          <p className="text-[var(--muted)] mt-4">
            axel creates symlinks from your centralized skills to Claude&apos;s expected location.
            This happens for each supported AI (Claude, Codex, OpenCode, Antigravity) - same skills, different destinations.
          </p>
          <p className="text-[var(--muted)] mt-2">
            When the session ends, axel cleans up the symlinks automatically. Your skills stay in one place,
            portable across all LLMs.
          </p>
        </Step>
      </section>

      <section className="mb-16">
        <h2 id="part-2-grid-layouts" className="text-2xl font-bold text-[var(--foreground)] mb-6">Part 2: Grid Layouts</h2>
        <p className="text-[var(--muted)] mb-6">
          Now let&apos;s set up a proper workspace with tmux. Grid layouts define how your panes are arranged.
        </p>

        <Step number={4} title="Add a Grid Layout">
          <p className="text-[var(--muted)] mb-4">
            Update your manifest to include Claude on the left,
            and two custom panes stacked on the right:
          </p>
          <YamlBlock filename="AXEL.md">
{`workspace: my-project

layouts:
  panes:
    - type: claude
      skills:
        - "*"
    - type: shell
    - type: server
      command: npm run dev

  grids:
    default:
      type: tmux
      claude:
        col: 0
        row: 0
      shell:
        col: 1
        row: 0
        color: yellow
      server:
        col: 1
        row: 1
        color: blue`}
          </YamlBlock>
        </Step>

        <Step number={5} title="Launch the Workspace">
          <p className="text-[var(--muted)] mb-4">
            Start your full workspace:
          </p>
          <CommandBlock>axel</CommandBlock>
          <p className="text-[var(--muted)] mt-4 mb-4">
            axel creates a tmux session named after your project, with all panes arranged according to your grid:
          </p>
          <div className="rounded-xl border border-[var(--border)] bg-zinc-900 dark:bg-black overflow-hidden">
            {/* Terminal header */}
            <div className="flex items-center gap-2 border-b border-zinc-800 bg-zinc-800/50 dark:bg-zinc-900/50 px-4 py-3">
              <div className="flex gap-1.5">
                <div className="h-3 w-3 rounded-full bg-red-500/80" />
                <div className="h-3 w-3 rounded-full bg-yellow-500/80" />
                <div className="h-3 w-3 rounded-full bg-green-500/80" />
              </div>
            </div>
            {/* Terminal panes */}
            <div className="p-2">
              <div className="flex h-64 gap-1">
                {/* Left pane - Claude */}
                <div className="flex-1 rounded border-2 border-purple-400/60 bg-purple-500/10 p-3 flex flex-col">
                  <div className="text-xs font-mono text-purple-400 mb-2">claude</div>
                  <div className="flex-1 font-mono text-xs text-zinc-500">
                    <span className="text-purple-400">❯</span> waiting for input...
                  </div>
                </div>
                {/* Right column */}
                <div className="flex-1 flex flex-col gap-1">
                  {/* Top right - Shell */}
                  <div className="flex-1 rounded border-2 border-yellow-400/60 bg-yellow-500/10 p-3 flex flex-col">
                    <div className="text-xs font-mono text-yellow-400 mb-2">shell</div>
                    <div className="flex-1 font-mono text-xs text-zinc-500">
                      <span className="text-yellow-400">$</span> _
                    </div>
                  </div>
                  {/* Bottom right - Server */}
                  <div className="flex-1 rounded border-2 border-blue-400/60 bg-blue-500/10 p-3 flex flex-col">
                    <div className="text-xs font-mono text-blue-400 mb-2">server</div>
                    <div className="flex-1 font-mono text-xs text-zinc-500">
                      <span className="text-emerald-400">▶</span> npm run dev
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          {/* Why tmux panel */}
          <div className="mt-6 rounded-lg border border-[var(--accent)]/20 bg-[var(--accent-dark)]/50 p-4">
            <h4 className="text-sm font-semibold text-[var(--accent)] mb-2">Why tmux?</h4>
            <p className="text-[var(--muted)] text-sm">
              tmux sessions persist even when you close your terminal. If you accidentally close a window or disconnect,
              just run <code className="text-[var(--accent)] font-medium text-xs">axel</code> again to reattach.
              Your Claude conversation, running servers, and shell history are all preserved.
            </p>
          </div>
        </Step>

        <Step number={6} title="Killing a Workspace">
          <p className="text-[var(--muted)] mb-4">
            When you&apos;re done, kill the workspace to clean up skills and terminate the tmux session:
          </p>
          <CommandBlock>axel -k my-project</CommandBlock>
          <p className="text-[var(--muted)] mt-4 mb-4">
            axel cleans up symlinks and restores your environment:
          </p>
          <div className="bg-[var(--card)] rounded-lg border border-[var(--border)] p-4 font-mono text-sm">
            <div className="text-[var(--muted)]">$ axel -k my-project</div>
            <div className="text-amber-600 dark:text-amber-400 mt-2">Stopping workspace my-project...</div>
            <div className="text-[var(--muted)] mt-2">Cleaning skills...</div>
            <div className="text-[var(--muted)] ml-2">→ removed .claude/skills/web-developer</div>
            <div className="text-[var(--muted)] ml-2">→ removed .claude/skills/rust-developer</div>
            <div className="text-[var(--muted)] mt-2">Killing tmux session...</div>
            <div className="text-emerald-600 dark:text-emerald-400 mt-2">✔ Workspace my-project terminated</div>
          </div>
        </Step>

        <Step number={7} title="Complex Layouts">
          <p className="text-[var(--muted)] mb-4">
            For fullstack development, you can create elaborate layouts with multiple AIs and services:
          </p>
          <YamlBlock filename="AXEL.md">
{`workspace: fullstack

layouts:
  panes:
    - type: claude
      skills: ["*"]
    - type: codex
      skills: ["*"]
    - type: frontend
      command: pnpm dev
    - type: backend
      command: cargo watch -x run
    - type: logs_fe
      command: tail -f logs/frontend.log
    - type: logs_be
      command: tail -f logs/backend.log

  grids:
    default:
      type: tmux
      claude:
        col: 0
        row: 0
      frontend:
        col: 0
        row: 1
      logs_fe:
        col: 1
        row: 1
        color: gray
      codex:
        col: 2
        row: 0
      backend:
        col: 2
        row: 1
      logs_be:
        col: 3
        row: 1
        color: gray`}
          </YamlBlock>
          <p className="text-[var(--muted)] mt-4 mb-4">
            This creates a workspace with two AIs working on different parts of your stack:
          </p>
          <div className="rounded-xl border border-[var(--border)] bg-zinc-900 dark:bg-black overflow-hidden">
            {/* Terminal header */}
            <div className="flex items-center gap-2 border-b border-zinc-800 bg-zinc-800/50 dark:bg-zinc-900/50 px-4 py-3">
              <div className="flex gap-1.5">
                <div className="h-3 w-3 rounded-full bg-red-500/80" />
                <div className="h-3 w-3 rounded-full bg-yellow-500/80" />
                <div className="h-3 w-3 rounded-full bg-green-500/80" />
              </div>
            </div>
            {/* Terminal panes */}
            <div className="p-2">
              <div className="flex h-72 gap-1">
                {/* Left side - Claude + Frontend + Logs */}
                <div className="flex-1 flex flex-col gap-1">
                  {/* Claude */}
                  <div className="flex-1 rounded border-2 border-purple-400/60 bg-purple-500/10 p-2 flex flex-col">
                    <div className="text-xs font-mono text-purple-400 mb-1">claude</div>
                    <div className="flex-1 font-mono text-[10px] text-zinc-500">
                      <span className="text-purple-400">❯</span> frontend tasks...
                    </div>
                  </div>
                  {/* Frontend + Logs row */}
                  <div className="flex-1 flex gap-1">
                    <div className="flex-1 rounded border-2 border-yellow-400/60 bg-yellow-500/10 p-2 flex flex-col">
                      <div className="text-xs font-mono text-yellow-400 mb-1">frontend</div>
                      <div className="font-mono text-[10px] text-zinc-500">
                        <span className="text-emerald-400">▶</span> pnpm dev
                      </div>
                    </div>
                    <div className="flex-1 rounded border-2 border-zinc-600/60 bg-zinc-500/10 p-2 flex flex-col">
                      <div className="text-xs font-mono text-zinc-400 mb-1">logs_fe</div>
                      <div className="font-mono text-[10px] text-zinc-500">
                        [info] ready
                      </div>
                    </div>
                  </div>
                </div>
                {/* Right side - Codex + Backend + Logs */}
                <div className="flex-1 flex flex-col gap-1">
                  {/* Codex */}
                  <div className="flex-1 rounded border-2 border-green-400/60 bg-green-500/10 p-2 flex flex-col">
                    <div className="text-xs font-mono text-green-400 mb-1">codex</div>
                    <div className="flex-1 font-mono text-[10px] text-zinc-500">
                      <span className="text-green-400">❯</span> backend tasks...
                    </div>
                  </div>
                  {/* Backend + Logs row */}
                  <div className="flex-1 flex gap-1">
                    <div className="flex-1 rounded border-2 border-blue-400/60 bg-blue-500/10 p-2 flex flex-col">
                      <div className="text-xs font-mono text-blue-400 mb-1">backend</div>
                      <div className="font-mono text-[10px] text-zinc-500">
                        <span className="text-emerald-400">▶</span> cargo watch
                      </div>
                    </div>
                    <div className="flex-1 rounded border-2 border-zinc-600/60 bg-zinc-500/10 p-2 flex flex-col">
                      <div className="text-xs font-mono text-zinc-400 mb-1">logs_be</div>
                      <div className="font-mono text-[10px] text-zinc-500">
                        [info] listening
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Step>
      </section>

      <section className="pt-8 border-t border-[var(--border)]">
        <h2 id="next-steps" className="text-2xl font-bold text-[var(--foreground)] mb-4">Next Steps</h2>
        <ul className="space-y-3 text-[var(--muted)]">
          <li>
            Learn all available{' '}
            <a href="/commands" className="text-[var(--accent)] hover:underline">
              CLI commands
            </a>
          </li>
          <li>
            Explore the full{' '}
            <a href="/manifest-file" className="text-[var(--accent)] hover:underline">
              manifest file reference
            </a>
          </li>
          <li>
            Learn about{' '}
            <a href="/skills" className="text-[var(--accent)] hover:underline">
              portable skills
            </a>
          </li>
          <li>
            Use{' '}
            <a href="/commands#git-worktrees" className="text-[var(--accent)] hover:underline">
              git worktrees
            </a>
            {' '}for parallel branch development
          </li>
        </ul>
      </section>
      </DocsBody>
    </DocsPage>
  );
}
