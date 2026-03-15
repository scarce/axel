import type { Metadata } from "next";
import { CommandBlock, YamlBlock } from "@/components/code";
import { DocsPage, DocsBody } from "fumadocs-ui/page";

export const metadata: Metadata = {
  title: "Sessions",
  description:
    "Persistent tmux sessions, git worktrees, and parallel development workflows",
};

const toc = [
  { title: "Why tmux?", url: "#why-tmux", depth: 2 },
  { title: "Session Lifecycle", url: "#session-lifecycle", depth: 2 },
  { title: "Managing Sessions", url: "#managing-sessions", depth: 2 },
  { title: "Git Worktrees", url: "#git-worktrees", depth: 2 },
  { title: "Parallel Development", url: "#parallel-development", depth: 2 },
];

export default function SessionsPage() {
  return (
    <DocsPage toc={toc}>
      <DocsBody>
        <h1 className="text-4xl font-bold text-[var(--foreground)] mb-4">
          Sessions
        </h1>
        <p className="text-[var(--muted)] mb-8 text-lg">
          axel uses tmux to create persistent, reproducible workspaces. Combined
          with git worktrees, you can run multiple branches simultaneously—each
          with its own isolated environment.
        </p>

        <section className="mb-12">
          <h2
            id="why-tmux"
            className="text-2xl font-bold text-[var(--foreground)] mb-4"
          >
            Why tmux?
          </h2>
          <div className="rounded-xl border border-[var(--border)] bg-gradient-to-br from-[var(--accent-dark)]/50 to-[var(--card)]/50 p-6 mb-6">
            <p className="text-[var(--foreground)]/80 leading-relaxed mb-4">
              <strong className="text-[var(--accent)]">
                Sessions persist.
              </strong>{" "}
              Close your terminal by mistake, your IDE crashed and ran out of
              memory—your workspace survives. Just run{" "}
              <code className="text-[var(--accent)] font-medium">axel</code>{" "}
              again to reattach. Your Claude conversation, running servers, and
              shell history are all preserved.
            </p>
            <p className="text-[var(--foreground)]/80 leading-relaxed">
              <strong className="text-[var(--accent)]">
                Layouts are reproducible.
              </strong>{" "}
              Define your pane arrangement once in YAML. Every time you launch,
              you get the exact same setup—Claude on the left, servers on the
              right, logs at the bottom. No manual window management.
            </p>
          </div>
        </section>

        <section className="mb-12">
          <h2
            id="session-lifecycle"
            className="text-2xl font-bold text-[var(--foreground)] mb-4"
          >
            Session Lifecycle
          </h2>
          <p className="text-[var(--muted)] mb-4">
            When you run{" "}
            <code className="text-[var(--accent)] font-medium">axel</code>, it
            creates a tmux session named after your workspace:
          </p>

          <div className="space-y-6">
            <div className="flex gap-4">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--accent-dark)] text-[var(--accent)] text-sm font-bold border border-[var(--accent)]/20">
                1
              </div>
              <div>
                <h4 className="font-semibold text-[var(--foreground)] mb-1">
                  Launch
                </h4>
                <p className="text-[var(--muted)] text-sm">
                  axel creates the tmux session, installs agent symlinks, and
                  spawns all configured panes.
                </p>
                <CommandBlock>axel</CommandBlock>
              </div>
            </div>

            <div className="flex gap-4">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--accent-dark)] text-[var(--accent)] text-sm font-bold border border-[var(--accent)]/20">
                2
              </div>
              <div>
                <h4 className="font-semibold text-[var(--foreground)] mb-1">
                  Detach
                </h4>
                <p className="text-[var(--muted)] text-sm">
                  Press{" "}
                  <code className="text-[var(--accent)] font-medium">
                    Ctrl+b d
                  </code>{" "}
                  to detach. The session keeps running in the background.
                </p>
              </div>
            </div>

            <div className="flex gap-4">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--accent-dark)] text-[var(--accent)] text-sm font-bold border border-[var(--accent)]/20">
                3
              </div>
              <div>
                <h4 className="font-semibold text-[var(--foreground)] mb-1">
                  Reattach
                </h4>
                <p className="text-[var(--muted)] text-sm">
                  Run axel again to reattach to an existing session. No restart,
                  no lost state.
                </p>
                <CommandBlock>axel</CommandBlock>
              </div>
            </div>

            <div className="flex gap-4">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--accent-dark)] text-[var(--accent)] text-sm font-bold border border-[var(--accent)]/20">
                4
              </div>
              <div>
                <h4 className="font-semibold text-[var(--foreground)] mb-1">
                  Kill
                </h4>
                <p className="text-[var(--muted)] text-sm">
                  When you&apos;re done, kill the session to clean up agents and
                  terminate processes.
                </p>
                <CommandBlock>axel -k my-project</CommandBlock>
              </div>
            </div>
          </div>
        </section>

        <section className="mb-12">
          <h2
            id="managing-sessions"
            className="text-2xl font-bold text-[var(--foreground)] mb-4"
          >
            Managing Sessions
          </h2>
          <p className="text-[var(--muted)] mb-6">
            Use{" "}
            <code className="text-[var(--accent)] font-medium">
              axel session
            </code>{" "}
            commands to manage your workspaces:
          </p>

          <div className="space-y-4">
            <div className="rounded-xl border border-[var(--border)] bg-[var(--card)]/50 p-5">
              <code className="text-[var(--accent)] font-mono text-base font-medium">
                axel session list
              </code>
              <p className="text-[var(--muted)] text-sm mt-2 mb-3">
                List all running axel sessions with their status and working
                directory.
              </p>
              <CommandBlock>axel session list</CommandBlock>
            </div>

            <div className="rounded-xl border border-[var(--border)] bg-[var(--card)]/50 p-5">
              <code className="text-[var(--accent)] font-mono text-base font-medium">
                axel session join &lt;name&gt;
              </code>
              <p className="text-[var(--muted)] text-sm mt-2 mb-3">
                Attach to an existing session. If already in tmux, switches to
                that session.
              </p>
              <CommandBlock>axel session join my-project</CommandBlock>
            </div>

            <div className="rounded-xl border border-[var(--border)] bg-[var(--card)]/50 p-5">
              <code className="text-[var(--accent)] font-mono text-base font-medium">
                axel session kill [name]
              </code>
              <p className="text-[var(--muted)] text-sm mt-2 mb-3">
                Kill a session by name, or the current session if no name is
                provided.
              </p>
              <CommandBlock>axel session kill my-project</CommandBlock>
            </div>
          </div>
        </section>

        <section className="mb-12">
          <h2
            id="git-worktrees"
            className="text-2xl font-bold text-[var(--foreground)] mb-4"
          >
            Git Worktrees
          </h2>
          <p className="text-[var(--muted)] mb-4">
            Git worktrees let you check out multiple branches simultaneously in
            separate directories. axel integrates with worktrees so each branch
            gets its own isolated workspace.
          </p>

          <div className="rounded-xl border border-[var(--border)] bg-[var(--card)]/50 p-6 mb-6">
            <h4 className="font-semibold text-[var(--foreground)] mb-3">
              The problem with branches
            </h4>
            <p className="text-[var(--muted)] text-sm mb-4">
              Traditional git workflows force you to stash, switch, and
              context-switch constantly. Working on a feature when a hotfix
              comes in? Stash everything, switch branches, fix, switch back, pop
              stash. Painful.
            </p>
            <h4 className="font-semibold text-[var(--foreground)] mb-3">
              The worktree solution
            </h4>
            <p className="text-[var(--muted)] text-sm">
              With worktrees, each branch lives in its own directory. No
              stashing, no switching. Run Claude on your feature branch in one
              terminal while fixing a bug on main in another.
            </p>
          </div>

          <p className="text-[var(--muted)] mb-4">
            Launch a workspace in a worktree with the{" "}
            <code className="text-[var(--accent)] font-medium">-w</code> flag:
          </p>
          <CommandBlock>axel -w feat/auth</CommandBlock>

          <p className="text-[var(--muted)] mt-4 mb-4">This command:</p>
          <ul className="list-disc list-inside text-[var(--muted)] space-y-2 mb-6">
            <li>
              Creates a worktree for{" "}
              <code className="text-[var(--accent)] font-medium">
                feat/auth
              </code>{" "}
              if it doesn&apos;t exist
            </li>
            <li>Creates the branch from your default branch if needed</li>
            <li>
              Symlinks your{" "}
              <code className="text-[var(--accent)] font-medium">AXEL.md</code>{" "}
              to the worktree
            </li>
            <li>Launches the workspace from the worktree directory</li>
          </ul>

          <p className="text-[var(--muted)] mb-4">
            Directory structure after running the command:
          </p>
          <div className="bg-[var(--card)] rounded-lg border border-[var(--border)] p-4 font-mono text-sm">
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

          <p className="text-[var(--muted)] mt-6 mb-4">
            To clean up a worktree when you&apos;re done:
          </p>
          <CommandBlock>
            axel -w feat/auth -k myproject-feat-auth --prune
          </CommandBlock>
        </section>

        <section>
          <h2
            id="parallel-development"
            className="text-2xl font-bold text-[var(--foreground)] mb-4"
          >
            Parallel Development
          </h2>
          <p className="text-[var(--muted)] mb-6">
            Combine sessions and worktrees for true parallel development. Run
            multiple workspaces simultaneously, each with its own branch and
            Claude instance.
          </p>

          <div className="rounded-xl border border-[var(--border)] bg-zinc-900 dark:bg-black overflow-hidden mb-6">
            {/* Terminal header */}
            <div className="flex items-center gap-2 border-b border-zinc-800 bg-zinc-800/50 dark:bg-zinc-900/50 px-4 py-3">
              <div className="flex gap-1.5">
                <div className="h-3 w-3 rounded-full bg-red-500/80" />
                <div className="h-3 w-3 rounded-full bg-yellow-500/80" />
                <div className="h-3 w-3 rounded-full bg-green-500/80" />
              </div>
            </div>
            {/* Terminal content */}
            <div className="p-4 font-mono text-sm">
              <div className="text-[var(--muted)]">
                # Terminal 1: Feature work
              </div>
              <div className="mb-3">
                <span className="text-purple-400">❯</span>
                <span className="text-zinc-300"> axel -w feat/auth</span>
              </div>
              <div className="text-[var(--muted)]">
                # Terminal 2: Bug fix on main
              </div>
              <div className="mb-3">
                <span className="text-purple-400">❯</span>
                <span className="text-zinc-300"> axel</span>
              </div>
              <div className="text-[var(--muted)]">
                # Terminal 3: Another feature
              </div>
              <div>
                <span className="text-purple-400">❯</span>
                <span className="text-zinc-300"> axel -w feat/payments</span>
              </div>
            </div>
          </div>

          <p className="text-[var(--muted)] mb-4">
            List all your running sessions to see what&apos;s active:
          </p>
          <CommandBlock>axel session list</CommandBlock>

          <div className="mt-6 rounded-lg border border-[var(--accent)]/20 bg-[var(--accent-dark)]/50 p-4">
            <h4 className="text-sm font-semibold text-[var(--accent)] mb-2">
              Pro tip
            </h4>
            <p className="text-[var(--muted)] text-sm">
              Each worktree session gets a unique name based on the branch
              (e.g.,{" "}
              <code className="text-[var(--accent)] font-medium text-xs">
                myproject-feat-auth
              </code>
              ). Use{" "}
              <code className="text-[var(--accent)] font-medium text-xs">
                axel session join
              </code>{" "}
              to quickly switch between them.
            </p>
          </div>
        </section>
      </DocsBody>
    </DocsPage>
  );
}
