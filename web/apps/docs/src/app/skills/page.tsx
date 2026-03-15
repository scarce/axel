import type { Metadata } from 'next';
import { CommandBlock, YamlBlock } from '@/components/code';
import { DocsPage, DocsBody } from 'fumadocs-ui/page';

export const metadata: Metadata = {
  title: 'Skills',
  description: 'Portable AI skills that work across LLMs - write once, deploy anywhere',
};

const toc = [
  { title: 'How Skills Work', url: '#how-skills-work', depth: 2 },
  { title: 'Skill Locations', url: '#skill-locations', depth: 2 },
  { title: 'Skill File Structure', url: '#skill-file-structure', depth: 2 },
  { title: 'Skill Commands', url: '#skill-commands', depth: 2 },
  { title: 'Using Skills in Workspaces', url: '#using-skills-in-workspaces', depth: 2 },
  { title: 'How Symlinks Work', url: '#how-symlinks-work', depth: 2 },
];

function CommandCard({
  command,
  description,
  example,
}: {
  command: string;
  description: string;
  example: string;
}) {
  return (
    <div className="rounded-xl border border-[var(--border)] bg-[var(--card)]/50 p-5 mb-4">
      <code className="text-[var(--accent)] font-mono text-base font-medium">{command}</code>
      <p className="text-[var(--muted)] text-sm mt-2 mb-3">{description}</p>
      <CommandBlock>{example}</CommandBlock>
    </div>
  );
}

export default function SkillsPage() {
  return (
    <DocsPage toc={toc}>
      <DocsBody>
        <h1 className="text-4xl font-bold text-[var(--foreground)] mb-4">Skills</h1>
      <p className="text-[var(--muted)] mb-8 text-lg">
        Write your prompts once, deploy them to any LLM. Skills are markdown files that axel automatically
        symlinks to each tool&apos;s expected location.
      </p>

      <section className="mb-12">
        <h2 id="how-skills-work" className="text-2xl font-bold text-[var(--foreground)] mb-4">How Skills Work</h2>
        <div className="rounded-xl border border-[var(--border)] bg-gradient-to-br from-[var(--accent-dark)]/50 to-[var(--card)]/50 p-6 mb-6">
          <p className="text-[var(--foreground)]/80 leading-relaxed mb-4">
            Each LLM tool expects skill files in different locations. Claude Code looks for{' '}
            <code className="text-[var(--accent)] font-medium">CLAUDE.md</code> or{' '}
            <code className="text-[var(--accent)] font-medium">.claude/skills/</code>,
            Codex uses <code className="text-[var(--accent)] font-medium">AGENTS.md</code>,
            and others have their own conventions.
          </p>
          <p className="text-[var(--foreground)]/80 leading-relaxed">
            axel solves this by letting you store skills in a single location and automatically
            creating the appropriate symlinks when you launch an assistant via axel.
          </p>
        </div>
      </section>

      <section className="mb-12">
        <h2 id="skill-locations" className="text-2xl font-bold text-[var(--foreground)] mb-4">Skill Locations</h2>
        <p className="text-[var(--muted)] mb-4">Skills can be stored in two locations:</p>
        <ul className="list-disc list-inside text-[var(--muted)] space-y-2 mb-6">
          <li>
            <strong className="text-[var(--foreground)]">Local</strong> -{' '}
            <code className="text-[var(--accent)] font-medium">./skills/</code> in your workspace
          </li>
          <li>
            <strong className="text-[var(--foreground)]">Global</strong> -{' '}
            <code className="text-[var(--accent)] font-medium">~/.config/axel/skills/</code> shared across workspaces
          </li>
        </ul>
        <p className="text-[var(--muted)]">
          Local skills take precedence. You can fork global skills to customize them per-project,
          or link them to keep them in sync.
        </p>
      </section>

      <section className="mb-12">
        <h2 id="skill-file-structure" className="text-2xl font-bold text-[var(--foreground)] mb-4">Skill File Structure</h2>
        <p className="text-[var(--muted)] mb-4">
          Skills are markdown files with optional YAML frontmatter:
        </p>
        <YamlBlock filename="skills/rust-developer/SKILL.md">
{`---
name: rust-developer
description: Expert Rust developer assistance
---

# Rust Developer

You are an expert Rust developer.

## Guidelines

- Write idiomatic Rust code
- Use proper error handling with Result and Option
- Prefer zero-cost abstractions
- Follow the Rust API guidelines`}
        </YamlBlock>
        <p className="text-[var(--muted)] mt-4">
          The directory name becomes the skill name. Use{' '}
          <code className="text-[var(--accent)] font-medium">axel skill list</code> to see all available skills.
        </p>
      </section>

      <section className="mb-12">
        <h2 id="skill-commands" className="text-2xl font-bold text-[var(--foreground)] mb-6">Skill Commands</h2>

        <CommandCard
          command="axel skill list"
          description="List all available skills, both local and global. Shows skill name, location, and description."
          example="axel skill list"
        />

        <CommandCard
          command="axel skill import <path>"
          description="Import a skill file or directory to the global skills directory. This is the recommended way to add existing skills to axel."
          example="axel skill import ./.claude/skills/web-developer"
        />

        <CommandCard
          command="axel skill new [name]"
          description="Create a new skill. If no name is provided, you will be prompted."
          example="axel skill new code-reviewer"
        />

        <CommandCard
          command="axel skill fork <name>"
          description="Copy a global skill to your local workspace. Use this when you need to customize a shared skill for this project."
          example="axel skill fork rust-developer"
        />

        <CommandCard
          command="axel skill link <name>"
          description="Create a symlink from a global skill to your local workspace. Changes to the global skill will be reflected in this workspace."
          example="axel skill link web-developer"
        />

        <CommandCard
          command="axel skill rm <name>"
          description="Remove a skill. If the skill exists in both locations, you'll be prompted to choose which one to remove."
          example="axel skill rm old-skill"
        />
      </section>

      <section className="mb-12">
        <h2 id="using-skills-in-workspaces" className="text-2xl font-bold text-[var(--foreground)] mb-4">Using Skills in Workspaces</h2>
        <p className="text-[var(--muted)] mb-4">
          Reference skills in your <code className="text-[var(--accent)] font-medium">AXEL.md</code>:
        </p>
        <YamlBlock filename="AXEL.md">
{`---
workspace: my-project

layouts:
  panes:
    - type: claude
      skills:
        - rust-developer
        - code-reviewer
      model: opus

    - type: codex
      skills:
        - rust-developer

  grids:
    default:
      type: tmux
      claude:
        col: 0
        row: 0
      codex:
        col: 1
        row: 0
---`}
        </YamlBlock>
        <p className="text-[var(--muted)] mt-4 mb-4">
          Use <code className="text-[var(--accent)] font-medium">&quot;*&quot;</code> to load all available skills:
        </p>
        <YamlBlock filename="AXEL.md">
{`---
layouts:
  panes:
    - type: claude
      skills:
        - "*"
---`}
        </YamlBlock>
        <p className="text-[var(--muted)] mt-4">
          When you launch an assistant via axel, it installs the skills to each tool&apos;s expected location
          and cleans them up when the session ends.
        </p>
      </section>

      <section>
        <h2 id="how-symlinks-work" className="text-2xl font-bold text-[var(--foreground)] mb-4">How Symlinks Work</h2>
        <p className="text-[var(--muted)] mb-4">
          When you run <code className="text-[var(--accent)] font-medium">axel claude</code>, axel:
        </p>
        <ul className="list-disc list-inside text-[var(--muted)] space-y-2 mb-6">
          <li>Creates skill directories in <code className="text-[var(--accent)] font-medium">.claude/skills/</code> with symlinked SKILL.md files</li>
          <li>Creates a CLAUDE.md symlink pointing to AXEL.md for project context</li>
          <li>Launches Claude Code with your skills pre-loaded</li>
          <li>Cleans up the symlinks when the session ends</li>
        </ul>
        <p className="text-[var(--muted)] mb-4">
          When you run <code className="text-[var(--accent)] font-medium">axel codex</code>, axel:
        </p>
        <ul className="list-disc list-inside text-[var(--muted)] space-y-2 mb-6">
          <li>Creates skill directories in <code className="text-[var(--accent)] font-medium">.codex/skills/</code> with symlinked SKILL.md files</li>
          <li>Creates an AGENTS.md symlink pointing to AXEL.md for project context</li>
          <li>Launches Codex with your skills available via <code className="text-[var(--accent)] font-medium">/skills</code></li>
          <li>Cleans up the symlinks when the session ends</li>
        </ul>
        <p className="text-[var(--muted)]">
          The same process applies for OpenCode (AGENTS.md), Antigravity, and other supported assistants—each gets skills installed in their expected location.
        </p>
      </section>
      </DocsBody>
    </DocsPage>
  );
}
