import type { Metadata } from 'next';
import { YamlBlock } from '@/components/code';
import { DocsPage, DocsBody } from 'fumadocs-ui/page';

export const metadata: Metadata = {
  title: 'Manifest File',
  description: 'AXEL.md reference - configure workspaces, skills, and terminal layouts',
};

const toc = [
  { title: 'File Location', url: '#file-location', depth: 2 },
  { title: 'Complete Example', url: '#complete-example', depth: 2 },
  { title: 'Top-Level Fields', url: '#top-level-fields', depth: 2 },
  { title: 'Pane Configuration', url: '#pane-configuration', depth: 2 },
  { title: 'Grid Layouts', url: '#grid-layouts', depth: 2 },
  { title: 'Colors', url: '#colors', depth: 2 },
];

function ConfigField({
  name,
  type,
  required,
  description,
  children,
}: {
  name: string;
  type: string;
  required?: boolean;
  description: string;
  children?: React.ReactNode;
}) {
  return (
    <div className="rounded-xl border border-[var(--border)] bg-[var(--card)]/50 p-5 mb-4">
      <div className="flex items-center gap-3 mb-2">
        <code className="text-[var(--accent)] font-mono text-base font-medium">{name}</code>
        <span className="text-xs text-[var(--muted)] bg-[var(--background)] px-2 py-0.5 rounded">{type}</span>
        {required && <span className="text-xs text-amber-600 dark:text-amber-400 bg-amber-500/10 px-2 py-0.5 rounded">required</span>}
      </div>
      <p className="text-[var(--muted)] text-sm">{description}</p>
      {children}
    </div>
  );
}

export default function ManifestFilePage() {
  return (
    <DocsPage toc={toc}>
      <DocsBody>
        <h1 className="text-4xl font-bold text-[var(--foreground)] mb-4">Manifest File</h1>
      <p className="text-[var(--muted)] mb-8 text-lg">
        The <code className="text-[var(--accent)]">AXEL.md</code> file defines your workspace configuration:
        which panes to run, where to find skills, and how to lay out terminal grids. It combines YAML frontmatter with markdown content for project context.
      </p>

      <section className="mb-12">
        <h2 id="file-location" className="text-2xl font-bold text-[var(--foreground)] mb-4">File Location</h2>
        <p className="text-[var(--muted)] mb-4">
          axel walks up the directory tree looking for <code className="text-[var(--accent)]">AXEL.md</code>.
          You can also specify a path with the <code className="text-[var(--accent)]">-m</code> flag.
        </p>
      </section>

      <section className="mb-12">
        <h2 id="complete-example" className="text-2xl font-bold text-[var(--foreground)] mb-6">Complete Example</h2>
        <YamlBlock filename="AXEL.md">
{`---
workspace: my-project

skills:
  - path: ./skills
  - path: ~/.config/axel/skills

layouts:
  panes:
    - type: claude
      color: gray
      skills:
        - "*"
    - type: shell
      notes:
        - "$ axel -k my-project"

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
---

# Project Documentation

Your project context goes here. This content is symlinked as CLAUDE.md for Claude Code
or AGENTS.md for Codex/OpenCode, providing project-specific instructions to the AI.

## Architecture

Describe your project structure, key patterns, and conventions...`}
        </YamlBlock>
      </section>

      <section className="mb-12">
        <h2 id="top-level-fields" className="text-2xl font-bold text-[var(--foreground)] mb-6">Top-Level Fields</h2>

        <ConfigField name="workspace" type="string" required description="Workspace name. Used for the tmux session name and display." />

        <ConfigField name="skills" type="array" description="List of directories to search for skill files. First match wins for duplicate names.">
          <YamlBlock>
{`skills:
  - path: ./skills
  - path: ~/.config/axel/skills`}
          </YamlBlock>
        </ConfigField>

        <ConfigField name="layouts" type="object" required description="Contains panes (pane definitions) and grids (layout profiles)." />
      </section>

      <section className="mb-12">
        <h2 id="pane-configuration" className="text-2xl font-bold text-[var(--foreground)] mb-6">Pane Configuration</h2>
        <p className="text-[var(--muted)] mb-6">
          Each pane in the <code className="text-[var(--accent)]">layouts.panes</code> array defines a pane type.
          Built-in types: <code className="text-[var(--accent)]">claude</code>, <code className="text-[var(--accent)]">codex</code>,{' '}
          <code className="text-[var(--accent)]">opencode</code>, <code className="text-[var(--accent)]">antigravity</code>,{' '}
          <code className="text-[var(--accent)]">shell</code>. Any other type is treated as a custom command.
        </p>

        <h3 className="text-lg font-semibold text-[var(--foreground)] mb-4">AI Pane (claude, codex, opencode, antigravity)</h3>
        <YamlBlock>
{`layouts:
  panes:
    - type: claude
      color: purple
      skills:
        - "*"                    # all skills, or list specific names
      model: sonnet              # sonnet, opus, haiku
      prompt: "Your task..."     # initial prompt
      allowed_tools:             # restrict to specific tools
        - Read
        - Write
      disallowed_tools: []       # block specific tools
      args: []                   # additional CLI arguments`}
        </YamlBlock>

        <div className="mt-6 space-y-3 text-sm">
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">type</code>
            <span className="text-[var(--muted)]">Pane type: claude, codex, opencode, antigravity, shell, or custom name</span>
          </div>
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">color</code>
            <span className="text-[var(--muted)]">Pane color: purple, yellow, red, green, blue, gray, orange</span>
          </div>
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">skills</code>
            <span className="text-[var(--muted)]">Skill names to load. Use &quot;*&quot; for all skills.</span>
          </div>
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">model</code>
            <span className="text-[var(--muted)]">Model to use (e.g., sonnet, opus, haiku for Claude)</span>
          </div>
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">prompt</code>
            <span className="text-[var(--muted)]">Initial prompt to send when launching</span>
          </div>
        </div>

        <h3 className="text-lg font-semibold text-[var(--foreground)] mb-4 mt-8">Shell Pane</h3>
        <YamlBlock>
{`layouts:
  panes:
    - type: shell
      notes:
        - "$ axel -k my-project"`}
        </YamlBlock>

        <div className="mt-6 space-y-3 text-sm">
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">notes</code>
            <span className="text-[var(--muted)]">Notes displayed in the pane on startup</span>
          </div>
        </div>

        <h3 className="text-lg font-semibold text-[var(--foreground)] mb-4 mt-8">Custom Command Pane</h3>
        <YamlBlock>
{`layouts:
  panes:
    - type: server
      command: cargo watch -x run
      color: blue`}
        </YamlBlock>

        <div className="mt-6 space-y-3 text-sm">
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">command</code>
            <span className="text-[var(--muted)]">Command to execute in this pane</span>
          </div>
        </div>
      </section>

      <section className="mb-12">
        <h2 id="grid-layouts" className="text-2xl font-bold text-[var(--foreground)] mb-6">Grid Layouts</h2>
        <p className="text-[var(--muted)] mb-6">
          Grids define how panes are arranged in tmux. Each grid has a type and pane positions.
        </p>

        <YamlBlock>
{`layouts:
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
        height: 50

    solo:
      type: shell
      claude:
        col: 0
        row: 0`}
        </YamlBlock>

        <h3 className="text-lg font-semibold text-[var(--foreground)] mb-4 mt-6">Grid Types</h3>
        <ul className="space-y-3 text-[var(--muted)] mb-6">
          <li>
            <code className="text-[var(--accent)]">tmux</code> - Standard tmux session with panes (default)
          </li>
          <li>
            <code className="text-[var(--accent)]">tmux_cc</code> - iTerm2 tmux integration mode
          </li>
          <li>
            <code className="text-[var(--accent)]">shell</code> - No tmux, run first pane directly
          </li>
        </ul>

        <h3 className="text-lg font-semibold text-[var(--foreground)] mb-4">Cell Positioning</h3>
        <p className="text-[var(--muted)] mb-4">
          Each pane name in the grid maps to a pane type. Position with column/row grid:
        </p>
        <div className="space-y-3 text-sm">
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">col</code>
            <span className="text-[var(--muted)]">Column position (0, 1, 2...) - left to right</span>
          </div>
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">row</code>
            <span className="text-[var(--muted)]">Row position (0, 1, 2...) - top to bottom within column</span>
          </div>
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">width</code>
            <span className="text-[var(--muted)]">Column width percentage</span>
          </div>
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">height</code>
            <span className="text-[var(--muted)]">Row height percentage</span>
          </div>
          <div className="flex gap-3">
            <code className="text-[var(--accent)] shrink-0">color</code>
            <span className="text-[var(--muted)]">Override pane color for this grid</span>
          </div>
        </div>
      </section>

      <section>
        <h2 id="colors" className="text-2xl font-bold text-[var(--foreground)] mb-4">Colors</h2>
        <p className="text-[var(--muted)] mb-4">Available pane colors:</p>
        <div className="flex flex-wrap gap-2">
          <code className="text-purple-400 bg-purple-500/10 border border-purple-500/30 px-3 py-1.5 rounded text-sm">purple</code>
          <code className="text-yellow-400 bg-yellow-500/10 border border-yellow-500/30 px-3 py-1.5 rounded text-sm">yellow</code>
          <code className="text-red-400 bg-red-500/10 border border-red-500/30 px-3 py-1.5 rounded text-sm">red</code>
          <code className="text-green-400 bg-green-500/10 border border-green-500/30 px-3 py-1.5 rounded text-sm">green</code>
          <code className="text-blue-400 bg-blue-500/10 border border-blue-500/30 px-3 py-1.5 rounded text-sm">blue</code>
          <code className="text-zinc-400 bg-zinc-500/10 border border-zinc-500/30 px-3 py-1.5 rounded text-sm">gray</code>
          <code className="text-orange-400 bg-orange-500/10 border border-orange-500/30 px-3 py-1.5 rounded text-sm">orange</code>
        </div>
      </section>
      </DocsBody>
    </DocsPage>
  );
}
