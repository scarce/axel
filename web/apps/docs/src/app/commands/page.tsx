import type { Metadata } from "next";
import { CommandBlock } from "@/components/code";
import { DocsPage, DocsBody } from "fumadocs-ui/page";

export const metadata: Metadata = {
  title: "CLI Commands",
  description: "Complete reference for all axel CLI commands",
};

const toc = [
  { title: "Launching", url: "#launching", depth: 2 },
  { title: "Git Worktrees", url: "#git-worktrees", depth: 2 },
  { title: "Setup", url: "#setup", depth: 2 },
  { title: "Session Commands", url: "#session-commands", depth: 2 },
  { title: "Agent Commands", url: "#agent-commands", depth: 2 },
  { title: "Other", url: "#other", depth: 2 },
];

function CommandSection({
  command,
  description,
  usage,
  options,
  examples,
}: {
  command: string;
  description: string;
  usage: string;
  options?: { flag: string; description: string }[];
  examples?: { code: string; description: string }[];
}) {
  return (
    <div className="mb-12 pb-12 border-b border-[var(--border)] last:border-0">
      <h2 className="text-xl font-bold text-[var(--foreground)] mb-2">
        <code className="text-[var(--accent)]">{command}</code>
      </h2>
      <p className="text-[var(--muted)] mb-4">{description}</p>

      <h3 className="text-sm font-semibold text-[var(--muted)] uppercase tracking-wider mb-2">
        Usage
      </h3>
      <CommandBlock>{usage}</CommandBlock>

      {options && options.length > 0 && (
        <>
          <h3 className="text-sm font-semibold text-[var(--muted)] uppercase tracking-wider mb-3 mt-6">
            Options
          </h3>
          <div className="space-y-2">
            {options.map((opt) => (
              <div key={opt.flag} className="flex gap-4">
                <code className="text-[var(--accent)] font-medium text-sm shrink-0">
                  {opt.flag}
                </code>
                <span className="text-[var(--muted)] text-sm">
                  {opt.description}
                </span>
              </div>
            ))}
          </div>
        </>
      )}

      {examples && examples.length > 0 && (
        <>
          <h3 className="text-sm font-semibold text-[var(--muted)] uppercase tracking-wider mb-3 mt-6">
            Examples
          </h3>
          <div className="space-y-4">
            {examples.map((ex, i) => (
              <div key={i}>
                <p className="text-[var(--muted)] text-sm mb-2">
                  {ex.description}
                </p>
                <CommandBlock>{ex.code}</CommandBlock>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}

export default function CommandsPage() {
  return (
    <DocsPage toc={toc}>
      <DocsBody>
        <h1 className="text-4xl font-bold text-[var(--foreground)] mb-4">
          CLI Commands
        </h1>
        <p className="text-[var(--muted)] mb-12 text-lg">
          Complete reference for all axel CLI commands and options.
        </p>

        <h2
          id="launching"
          className="text-2xl font-bold text-[var(--foreground)] mb-6"
        >
          Launching
        </h2>

        <CommandSection
          command="axel"
          description="Launch a workspace from the current directory. axel walks up the directory tree looking for a AXEL.md file and launches the configured session."
          usage="axel [SHELL]"
          options={[
            {
              flag: "-m, --manifest-path <PATH>",
              description: "Path to AXEL.md (default: ./AXEL.md)",
            },
            {
              flag: "-p, --profile <PROFILE>",
              description: 'Terminal profile to use (default: "default")',
            },
            {
              flag: "-w, --worktree <BRANCH>",
              description: "Create/use git worktree for branch",
            },
          ]}
          examples={[
            {
              description: "Launch full workspace from current directory",
              code: "axel",
            },
            {
              description: "Launch a specific shell from the manifest",
              code: "axel claude",
            },
            {
              description: "Launch with a specific profile",
              code: "axel -p focus",
            },
            {
              description: "Launch in a git worktree",
              code: "axel -w feat/new-feature",
            },
            {
              description: "Launch from a custom manifest",
              code: "axel -m ./configs/dev.yaml",
            },
          ]}
        />

        <CommandSection
          command="axel -k, --kill <workspace>"
          description="Kill a running workspace session. Terminates all panes, closes the tmux session, and cleans up any agent symlinks that were created."
          usage="axel -k <workspace>"
          options={[
            {
              flag: "--keep-agents",
              description: "Keep agent symlinks instead of cleaning them up",
            },
            {
              flag: "--prune",
              description: "Also remove the git worktree (use with -w)",
            },
          ]}
          examples={[
            {
              description: "Kill a running workspace",
              code: "axel -k my-project",
            },
            {
              description: "Kill but keep agent files",
              code: "axel -k my-project --keep-agents",
            },
            {
              description: "Kill workspace and remove worktree",
              code: "axel -w feat/auth -k my-project-feat-auth --prune",
            },
          ]}
        />

        <h2
          id="git-worktrees"
          className="text-2xl font-bold text-[var(--foreground)] mb-6 mt-12"
        >
          Git Worktrees
        </h2>
        <p className="text-[var(--muted)] mb-6">
          Launch workspaces in isolated git worktrees for parallel branch
          development. Worktrees let you work on multiple branches
          simultaneously without stashing or switching.
        </p>

        <CommandSection
          command="axel -w, --worktree <branch>"
          description="Create or use a git worktree for the specified branch and launch the workspace from there. If the branch doesn't exist, it will be created from the default branch (main/master). The worktree is created as a sibling directory to your repository."
          usage="axel -w <branch>"
          options={[
            {
              flag: "-w, --worktree <BRANCH>",
              description: "Branch name to create/use worktree for",
            },
          ]}
          examples={[
            {
              description: "Create worktree and launch workspace",
              code: "axel -w feat/auth",
            },
            {
              description: "Launch specific shell in worktree",
              code: "axel -w feat/auth claude",
            },
            { description: "Use existing worktree", code: "axel -w feat/auth" },
          ]}
        />

        <div className="rounded-xl border border-[var(--border)] bg-[var(--card)]/50 p-6 mb-12">
          <h3 className="text-lg font-semibold text-[var(--foreground)] mb-4">
            How It Works
          </h3>
          <div className="space-y-4 text-[var(--muted)] text-sm">
            <p>
              When you run{" "}
              <code className="text-[var(--accent)] font-medium">
                axel -w feat/auth
              </code>
              :
            </p>
            <ol className="list-decimal list-inside space-y-2 ml-2">
              <li>
                If the branch exists locally or on remote, a worktree is created
                for it
              </li>
              <li>
                If the branch doesn&apos;t exist, it&apos;s created from your
                default branch
              </li>
              <li>
                Your{" "}
                <code className="text-[var(--accent)] font-medium">
                  AXEL.md
                </code>{" "}
                is symlinked to the worktree
              </li>
              <li>The workspace launches from the worktree directory</li>
            </ol>
            <div className="mt-4 p-4 bg-[var(--background)] rounded-lg font-mono text-xs">
              <div className="text-[var(--muted)]">~/code/</div>
              <div className="text-[var(--muted)]">
                ├── myproject/{" "}
                <span className="text-[var(--accent)]"># main repo</span>
              </div>
              <div className="text-[var(--muted)]">│ ├── AXEL.md</div>
              <div className="text-[var(--muted)]">│ └── src/</div>
              <div className="text-[var(--muted)]">
                └── myproject-feat-auth/{" "}
                <span className="text-[var(--accent)]"># worktree</span>
              </div>
              <div className="text-[var(--muted)]">
                {" "}
                ├── AXEL.md → ../myproject/AXEL.md
              </div>
              <div className="text-[var(--muted)]"> └── src/</div>
            </div>
          </div>
        </div>

        <h2
          id="setup"
          className="text-2xl font-bold text-[var(--foreground)] mb-6 mt-12"
        >
          Setup
        </h2>

        <CommandSection
          command="axel init"
          description="Initialize a axel workspace in the current directory. Creates a AXEL.md file with a basic configuration."
          usage="axel init"
          examples={[
            { description: "Initialize a new workspace", code: "axel init" },
          ]}
        />

        <CommandSection
          command="axel bootstrap"
          description="[Experimental] Scan your machine for existing agents and consolidate them using AI. We recommend using axel agent import for more control."
          usage="axel bootstrap"
          examples={[
            { description: "Auto-discover agents", code: "axel bootstrap" },
          ]}
        />

        <h2
          id="session-commands"
          className="text-2xl font-bold text-[var(--foreground)] mb-6 mt-12"
        >
          Session Commands
        </h2>
        <p className="text-[var(--muted)] mb-6">
          Manage running axel tmux sessions. List active workspaces, create new
          ones, or kill existing sessions.
        </p>

        <CommandSection
          command="axel session list"
          description="List all running axel sessions. Shows session name, working directory, window count, and attachment status."
          usage="axel session list"
          options={[
            {
              flag: "-a, --all",
              description: "Show all tmux sessions, not just axel sessions",
            },
          ]}
          examples={[
            { description: "List axel sessions", code: "axel session list" },
            {
              description: "List all tmux sessions",
              code: "axel session ls --all",
            },
          ]}
        />

        <CommandSection
          command="axel session new"
          description="Create a new workspace session. Equivalent to running `axel` or `axel <shell>`. Launches a workspace from the AXEL.md manifest in the current directory."
          usage="axel session new [SHELL]"
          examples={[
            { description: "Create new workspace", code: "axel session new" },
            {
              description: "Create with specific shell",
              code: "axel session new claude",
            },
          ]}
        />

        <CommandSection
          command="axel session join <name>"
          description="Attach to an existing axel or tmux session. If already inside tmux, switches to the target session."
          usage="axel session join <name>"
          examples={[
            {
              description: "Join a session",
              code: "axel session join my-project",
            },
          ]}
        />

        <CommandSection
          command="axel session kill"
          description="Kill a running workspace session. Equivalent to `axel -k <name>`. Terminates all panes, closes the tmux session, and cleans up agent symlinks."
          usage="axel session kill [NAME]"
          options={[
            {
              flag: "--keep-agents",
              description: "Keep agent symlinks instead of cleaning them up",
            },
          ]}
          examples={[
            { description: "Kill current session", code: "axel session kill" },
            {
              description: "Kill named session",
              code: "axel session kill my-project",
            },
            {
              description: "Kill but keep agents",
              code: "axel session kill my-project --keep-agents",
            },
          ]}
        />

        <h2
          id="agent-commands"
          className="text-2xl font-bold text-[var(--foreground)] mb-6 mt-12"
        >
          Agent Commands
        </h2>
        <p className="text-[var(--muted)] mb-6">
          Manage your portable agent files. See the{" "}
          <a href="/agents" className="text-[var(--accent)] hover:underline">
            Agents
          </a>{" "}
          documentation for more details.
        </p>

        <CommandSection
          command="axel agent list"
          description="List all available agents, both local (in current workspace) and global (in ~/.config/axel/agents)."
          usage="axel agent list"
          examples={[
            { description: "List all agents", code: "axel agent list" },
            { description: "Using the alias", code: "axel agents ls" },
          ]}
        />

        <CommandSection
          command="axel agent import <path>"
          description="Import an agent file or directory to the global agents directory. This makes the agent available across all your workspaces."
          usage="axel agent import <path>"
          examples={[
            {
              description: "Import a single agent file",
              code: "axel agent import ./.claude/agents/web-developer.md",
            },
            {
              description: "Import all agents from a directory",
              code: "axel agent import ./agents/",
            },
          ]}
        />

        <CommandSection
          command="axel agent new [name]"
          description="Create a new agent file. If no name is provided, you will be prompted."
          usage="axel agent new [name]"
          examples={[
            {
              description: "Create a new agent with name",
              code: "axel agent new code-reviewer",
            },
            { description: "Create interactively", code: "axel agent new" },
          ]}
        />

        <CommandSection
          command="axel agent fork <name>"
          description="Copy a global agent to the current workspace's local agents directory. This creates a local copy you can customize."
          usage="axel agent fork <name>"
          examples={[
            {
              description: "Fork a global agent locally",
              code: "axel agent fork web-developer",
            },
          ]}
        />

        <CommandSection
          command="axel agent link <name>"
          description="Create a symlink from the current workspace to a global agent. Changes to the global agent will be reflected locally."
          usage="axel agent link <name>"
          examples={[
            {
              description: "Link a global agent",
              code: "axel agent link code-reviewer",
            },
          ]}
        />

        <CommandSection
          command="axel agent rm <name>"
          description="Remove an agent file."
          usage="axel agent rm <name>"
          examples={[
            { description: "Remove an agent", code: "axel agent rm old-agent" },
          ]}
        />

        <h2
          id="other"
          className="text-2xl font-bold text-[var(--foreground)] mb-6 mt-12"
        >
          Other
        </h2>

        <CommandSection
          command="axel --version"
          description="Display the current version of axel."
          usage="axel --version"
          examples={[{ description: "Check version", code: "axel --version" }]}
        />

        <CommandSection
          command="axel --help"
          description="Display help information and a list of all available commands."
          usage="axel --help"
          examples={[{ description: "Show help", code: "axel --help" }]}
        />
      </DocsBody>
    </DocsPage>
  );
}
