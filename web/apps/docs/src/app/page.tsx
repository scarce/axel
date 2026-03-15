import Link from "next/link";
import { DocsPage, DocsBody } from "fumadocs-ui/page";

export const metadata = {
  title: {
    absolute: "Overview | axel docs",
  },
  description:
    "axel documentation - portable agents across LLMs, reproducible terminal workspaces",
};

function NavCard({
  title,
  description,
  href,
  icon,
}: {
  title: string;
  description: string;
  href: string;
  icon: React.ReactNode;
}) {
  return (
    <Link
      href={href}
      className="group relative overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--card)] p-5 transition-all hover:border-[var(--accent)]/30 hover:bg-[var(--accent-dark)]/30"
    >
      <div className="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
      <div className="relative flex items-start gap-4">
        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-[var(--accent-dark)] text-[var(--accent)] border border-[var(--accent)]/20">
          {icon}
        </div>
        <div>
          <h3 className="text-base font-semibold text-[var(--foreground)] mb-1 group-hover:text-[var(--accent)] transition-colors">
            {title}
          </h3>
          <p className="text-[var(--muted)] text-sm leading-relaxed">
            {description}
          </p>
        </div>
      </div>
      <div className="absolute right-4 top-1/2 -translate-y-1/2 text-[var(--border)] group-hover:text-[var(--accent)] group-hover:translate-x-1 transition-all">
        <svg
          className="h-5 w-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M9 5l7 7-7 7"
          />
        </svg>
      </div>
    </Link>
  );
}

function ConceptCard({
  title,
  description,
  icon,
}: {
  title: string;
  description: string;
  icon: React.ReactNode;
}) {
  return (
    <div className="relative overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--card)]/50 p-5">
      <div className="flex items-center gap-3 mb-3">
        <div className="flex h-8 w-8 items-center justify-center rounded-md bg-[var(--accent-dark)] text-[var(--accent)]">
          {icon}
        </div>
        <h4 className="text-base font-semibold text-[var(--foreground)]">
          {title}
        </h4>
      </div>
      <p className="text-[var(--muted)] text-sm leading-relaxed">
        {description}
      </p>
    </div>
  );
}

// Icons
const BookIcon = () => (
  <svg
    className="h-5 w-5"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
    />
  </svg>
);

const RocketIcon = () => (
  <svg
    className="h-5 w-5"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M15.59 14.37a6 6 0 01-5.84 7.38v-4.8m5.84-2.58a14.98 14.98 0 006.16-12.12A14.98 14.98 0 009.631 8.41m5.96 5.96a14.926 14.926 0 01-5.841 2.58m-.119-8.54a6 6 0 00-7.381 5.84h4.8m2.581-5.84a14.927 14.927 0 00-2.58 5.84m2.699 2.7c-.103.021-.207.041-.311.06a15.09 15.09 0 01-2.448-2.448 14.9 14.9 0 01.06-.312m-2.24 2.39a4.493 4.493 0 00-1.757 4.306 4.493 4.493 0 004.306-1.758M16.5 9a1.5 1.5 0 11-3 0 1.5 1.5 0 013 0z"
    />
  </svg>
);

const TerminalIcon = () => (
  <svg
    className="h-5 w-5"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v12a2.25 2.25 0 002.25 2.25z"
    />
  </svg>
);

const BotIcon = () => (
  <svg
    className="h-5 w-5"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M9.75 3.104v5.714a2.25 2.25 0 01-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 014.5 0m0 0v5.714c0 .597.237 1.17.659 1.591L19.8 15.3M14.25 3.104c.251.023.501.05.75.082M19.8 15.3l-1.57.393A9.065 9.065 0 0112 15a9.065 9.065 0 00-6.23.693L5 15.5m14.8-.2l.2 1.2a2.5 2.5 0 01-2.5 2.5h-11a2.5 2.5 0 01-2.5-2.5l.2-1.2"
    />
  </svg>
);

const CogIcon = () => (
  <svg
    className="h-5 w-5"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z"
    />
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
    />
  </svg>
);

const BlocksIcon = () => (
  <svg
    className="h-4 w-4"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M21 7.5l-9-5.25L3 7.5m18 0l-9 5.25m9-5.25v9l-9 5.25M3 7.5l9 5.25M3 7.5v9l9 5.25m0-9v9"
    />
  </svg>
);

const LayersIcon = () => (
  <svg
    className="h-4 w-4"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M6.429 9.75L2.25 12l4.179 2.25m0-4.5l5.571 3 5.571-3m-11.142 0L2.25 7.5 12 2.25l9.75 5.25-4.179 2.25m0 0L21.75 12l-4.179 2.25m0 0l4.179 2.25L12 21.75 2.25 16.5l4.179-2.25m11.142 0l-5.571 3-5.571-3"
    />
  </svg>
);

const LayoutIcon = () => (
  <svg
    className="h-4 w-4"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M3.75 6A2.25 2.25 0 016 3.75h2.25A2.25 2.25 0 0110.5 6v2.25a2.25 2.25 0 01-2.25 2.25H6a2.25 2.25 0 01-2.25-2.25V6zM3.75 15.75A2.25 2.25 0 016 13.5h2.25a2.25 2.25 0 012.25 2.25V18a2.25 2.25 0 01-2.25 2.25H6A2.25 2.25 0 013.75 18v-2.25zM13.5 6a2.25 2.25 0 012.25-2.25H18A2.25 2.25 0 0120.25 6v2.25A2.25 2.25 0 0118 10.5h-2.25a2.25 2.25 0 01-2.25-2.25V6zM13.5 15.75a2.25 2.25 0 012.25-2.25H18a2.25 2.25 0 012.25 2.25V18A2.25 2.25 0 0118 20.25h-2.25A2.25 2.25 0 0113.5 18v-2.25z"
    />
  </svg>
);

const toc = [
  { title: "Core Concepts", url: "#core-concepts", depth: 2 },
  { title: "Getting Started", url: "#getting-started", depth: 2 },
  { title: "Reference", url: "#reference", depth: 2 },
];

export default function DocsHomePage() {
  return (
    <DocsPage toc={toc}>
      <DocsBody>
        <h1 className="text-4xl font-bold text-[var(--foreground)] mb-4">
          axel
        </h1>
        <p className="text-xl text-[var(--muted)] mb-8 leading-relaxed max-w-2xl">
          A CLI for AI-assisted development workflows.
        </p>

        <div className="rounded-xl border border-[var(--border)] bg-gradient-to-br from-[var(--accent-dark)]/50 to-[var(--card)]/50 p-6 mb-12 max-w-2xl">
          <p className="text-[var(--foreground)]/80 leading-relaxed mb-4">
            <strong className="text-[var(--accent)]">Agent portability.</strong>{" "}
            Write your agents once, deploy them to any LLM. No more maintaining
            separate configs for Claude, Codex, and whatever ships next.
          </p>
          <p className="text-[var(--foreground)]/80 leading-relaxed">
            <strong className="text-[var(--accent)]">
              Terminal real estate.
            </strong>{" "}
            Define your workspace once, launch it with one command. Claude on
            the left, servers on the right, logs at the bottom. Reproducible and
            instant.
          </p>
        </div>

        <h2
          id="core-concepts"
          className="text-xl font-semibold text-[var(--foreground)] mb-4"
        >
          Core Concepts
        </h2>
        <div className="grid md:grid-cols-3 gap-3 mb-12">
          <ConceptCard
            icon={<BlocksIcon />}
            title="Agents"
            description="Markdown files with system prompts. Stored centrally, symlinked to each LLM's expected location."
          />
          <ConceptCard
            icon={<LayersIcon />}
            title="Shells"
            description="What runs in each pane: Claude, Codex, custom commands, or plain shell."
          />
          <ConceptCard
            icon={<LayoutIcon />}
            title="Profiles"
            description="Terminal layouts. Map shells to grid positions. Switch between different arrangements."
          />
        </div>

        <h2
          id="getting-started"
          className="text-xl font-semibold text-[var(--foreground)] mb-4"
        >
          Getting Started
        </h2>
        <div className="grid md:grid-cols-2 gap-3 mb-12">
          <NavCard
            icon={<BookIcon />}
            title="Installation"
            description="Install axel and set up tmux"
            href="/installation"
          />
          <NavCard
            icon={<RocketIcon />}
            title="Quick Start"
            description="Create your first workspace"
            href="/quick-start"
          />
        </div>

        <h2
          id="reference"
          className="text-xl font-semibold text-[var(--foreground)] mb-4"
        >
          Reference
        </h2>
        <div className="grid md:grid-cols-3 gap-3">
          <NavCard
            icon={<TerminalIcon />}
            title="CLI Commands"
            description="All available commands"
            href="/commands"
          />
          <NavCard
            icon={<BotIcon />}
            title="Agents"
            description="Portable AI prompts"
            href="/agents"
          />
          <NavCard
            icon={<CogIcon />}
            title="Manifest File"
            description="AXEL.md reference"
            href="/manifest-file"
          />
        </div>
      </DocsBody>
    </DocsPage>
  );
}
