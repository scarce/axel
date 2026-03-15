# axel

A CLI for AI-assisted development. Portable skills across LLMs. Reproducible terminal workspaces.

## The Problem

Vibe coding is here. LLMs are racing to top benchmarks with infinite funding behind them. You want to switch when one pulls ahead—but your skills are stuck in `.claude/` or `.codex/`.

Not using skills? You're leaving most of the LLM potential on the table. Copy-pasting them between tools? They drift out of sync.

Meanwhile, your terminal is chaos and your IDE crashes and keep losing context.

**axel fixes both and goes beyond.**

## Features

### Skill Portability

Write your skills once. axel symlinks them wherever they need to go.

```
skills/
  code-reviewer.md
  frontend-engineer.md
  security-auditor.md
```

![Skill Portability](docs/agents.png)


Switch LLMs by changing one line. No more copy-pasting between `.claude/commands/` and `.codex/agents/`.

### Reproducible Workspaces

One command. Your entire workspace materializes.

```yaml
# AXEL.md (frontmatter)
workspace: myproject

layouts:
  panes:
    - type: claude
      skills: ["*"]
    - type: custom
      name: backend
      command: npm run dev
      path: ./backend
    - type: custom
      name: frontend
      command: pnpm dev
      path: ./frontend

  grids:
    default:
      type: tmux
      claude:
        col: 0
        row: 0
      backend:
        col: 1
        row: 0
      frontend:
        col: 1
        row: 1
```

```bash
axel
```

![Tmux workspace layouts](docs/tmux-layout.png)

Claude on the left, servers on the right. Close everything, come back tomorrow, run `axel` again—exactly where you left off.

## Installation

```bash
curl -sL https://install.axel.md | bash
```

Or build from source:

```bash
cargo install --path crates/cli
```

### Prerequisites

- [tmux](https://github.com/tmux/tmux) for workspace management
- One or more AI coding assistants

### Supported LLMs

Skills can be dispatched to any of the following LLMs:

- [x] [Claude Code](https://claude.ai/code) - Anthropic
- [x] [Codex](https://openai.com/codex) - OpenAI
- [x] [OpenCode](https://opencode.ai) - Open source
- [x] [Antigravity](https://antigravityai.org) - Google

## Quick Start

> See the full [Quick Start guide](https://docs.axel.md/quick-start) for detailed instructions.

```bash
# Initialize a workspace in current directory
axel init

# Import your existing skills
axel skill import ./.claude/commands/web-developer.md
axel skill import ./skills/

# Launch a single AI pane (skills are symlinked automatically)
axel claude

# Or launch the full workspace
axel
```

> **Note:** `axel bootstrap` exists to auto-discover skills across your machine, but it's experimental. We recommend manually importing skills with `axel skill import <file|dir>` for more control.

## Usage

```bash
# Daily workflow
axel                          # Launch workspace from AXEL.md
axel -w feat/auth             # Launch in a git worktree
axel -k                       # Kill session and clean up

# Sessions
axel session list             # List running sessions
axel session join <name>      # Attach to a session
axel session kill <name>      # Kill a session

# Layouts
axel layout ls                # List panes and grids (JSON output)
axel layout ls --json         # Explicit JSON format

# Skills
axel skill list               # List all skills
axel skill import <path>      # Import from file or directory
axel skill new                # Create a new skill
axel skill fork <name>        # Copy global skill locally
axel skill link <name>        # Symlink global skill locally
```

See the [CLI Reference](https://docs.axel.md/commands) for all options.

## Configuration

### Layouts

Define panes and grid layouts in the `layouts` section:

```yaml
layouts:
  # Pane definitions - what runs in each pane
  panes:
    # AI assistants (built-in types: claude, codex, opencode, antigravity)
    - type: claude
      skills: ["code-reviewer", "frontend-engineer"]
      model: sonnet

    - type: codex
      skills: ["*"]  # All skills

    # Custom panes - use type: custom with unique names
    - type: custom
      name: shell
      notes:
        - "Free shell for commands"

    - type: custom
      name: backend
      path: ./backend
      command: npm run dev

    - type: custom
      name: frontend
      path: ./frontend
      command: pnpm dev

  # Grid layouts - how panes are arranged
  # Reference panes by their name (claude, codex, shell, backend, frontend)
  grids:
    default:
      type: tmux  # tmux, tmux_cc (iTerm2), or shell
      claude:
        col: 0
        row: 0
      backend:
        col: 1
        row: 0
        height: 50
      frontend:
        col: 1
        row: 1
```

### Skills

Centralize your skills:

```yaml
skills:
  - path: ./skills           # Local skills
  - path: ~/.config/axel/skills  # Global skills
```

## For Developers Who

- Run multiple AI coding assistants and are tired of maintaining separate skill configs
- Work across several repositories that need to be running simultaneously
- Want reproducible dev environments they can spin up instantly
- Believe their tooling should adapt to them, not the other way around

## Links

- [Website](https://axel.md)
- [Documentation](https://docs.axel.md)

## License

MIT
