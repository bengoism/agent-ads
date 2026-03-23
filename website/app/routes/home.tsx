import { Link } from "react-router";
import {
  engineOrder,
  engines,
  homeCLIExamples,
  homePromptCards,
  version,
} from "../content/site";
import { CodeBlock } from "../components/code-block";
import {
  BadgeList,
  btnSecondary,
  CopyButton,
  eyebrowCls,
  SectionHeader,
} from "../components/docs";

export const meta = () => [
  { title: "agent-ads | One CLI for every major ad platform" },
  {
    name: "description",
    content:
      "Read-only CLI for Meta, Google, TikTok, and Pinterest ad APIs. JSON to stdout. Built in Rust. Works with Claude Code.",
  },
];

function EngineIcon({ engineId }: { engineId: string }) {
  const cls = "w-5 h-5 shrink-0";
  switch (engineId) {
    case "meta":
      return (
        <svg className="w-7 h-7 shrink-0" viewBox="0 0 36 20" fill="currentColor" aria-hidden="true">
          <path d="M7.06 0C3.56 0 1.4 3.2.44 5.44c-.7 1.64-1.04 3.5-.44 5.3C.76 13.1 2.68 15 5.6 15c2.14 0 3.88-1.1 5.36-2.94L13 9.5l2.04 2.56C16.52 13.9 18.26 15 20.4 15c2.92 0 4.84-1.9 5.6-4.26.6-1.8.26-3.66-.44-5.3C24.6 3.2 22.44 0 18.94 0c-2.52 0-4.44 1.66-5.94 3.7-1.5-2.04-3.42-3.7-5.94-3.7zm.14 3.4c1.72 0 3.28 1.92 4.52 3.7L13 8.86l1.28-1.76c1.24-1.78 2.8-3.7 4.52-3.7 1.92 0 3.28 1.8 4.14 3.64.52 1.12.62 2.18.3 3.14-.48 1.38-1.56 2.22-3.04 2.22-1.14 0-2.24-.72-3.48-2.28L13 5.74l-3.72 4.38C8.04 11.68 6.94 12.4 5.8 12.4c-1.48 0-2.56-.84-3.04-2.22-.32-.96-.22-2.02.3-3.14C3.92 5.2 5.28 3.4 7.2 3.4z" />
        </svg>
      );
    case "google":
      return (
        <svg className={cls} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" />
          <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" />
          <path d="M5.84 14.09A6.97 6.97 0 0 1 5.47 12c0-.72.13-1.43.37-2.09V7.07H2.18A11 11 0 0 0 1 12c0 1.77.43 3.45 1.18 4.93l3.66-2.84z" />
          <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" />
        </svg>
      );
    case "tiktok":
      return (
        <svg className={cls} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path d="M19.59 6.69a4.83 4.83 0 0 1-3.77-4.25V2h-3.45v13.67a2.89 2.89 0 0 1-2.88 2.5 2.89 2.89 0 0 1 0-5.78c.27 0 .54.04.79.1v-3.5a6.37 6.37 0 0 0-.79-.05A6.34 6.34 0 0 0 3.15 15.3 6.34 6.34 0 0 0 9.49 21.6a6.34 6.34 0 0 0 6.34-6.34V9.39a8.16 8.16 0 0 0 4.76 1.52V7.48a4.82 4.82 0 0 1-1-.79z" />
        </svg>
      );
    case "pinterest":
      return (
        <svg className={cls} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path d="M12 0a12 12 0 0 0-4.37 23.17c-.1-.94-.2-2.4.04-3.44l1.4-5.96s-.36-.72-.36-1.78c0-1.66.97-2.9 2.17-2.9 1.02 0 1.52.77 1.52 1.7 0 1.03-.66 2.58-.99 4.01-.28 1.2.6 2.17 1.78 2.17 2.13 0 3.77-2.25 3.77-5.5 0-2.87-2.06-4.88-5.01-4.88-3.41 0-5.42 2.56-5.42 5.21 0 1.03.4 2.14.89 2.74a.36.36 0 0 1 .08.34l-.33 1.36c-.05.22-.18.27-.4.16-1.5-.7-2.44-2.9-2.44-4.67 0-3.8 2.76-7.3 7.96-7.3 4.18 0 7.43 2.98 7.43 6.96 0 4.15-2.62 7.5-6.25 7.5-1.22 0-2.37-.63-2.76-1.38l-.75 2.87c-.27 1.05-1.01 2.37-1.5 3.17A12 12 0 1 0 12 0z" />
        </svg>
      );
    default:
      return null;
  }
}

export default function HomeRoute() {
  return (
    <>
      {/* ── Version badges + Hero ────────────────────────── */}
      <section className="grid gap-6">
        <div className="flex flex-wrap gap-2">
          <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-accent/12 text-accent text-[0.72rem] font-bold tracking-[0.06em] uppercase">
            v{version} stable
          </span>
          <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-surface-highest/52 text-fg-muted text-[0.72rem] font-bold tracking-[0.06em] uppercase">
            Rust powered CLI
          </span>
          <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-surface-highest/52 text-fg-muted text-[0.72rem] font-bold tracking-[0.06em] uppercase">
            Works with Claude Code
          </span>
        </div>

        <div className="grid gap-6 max-w-[42rem]">
          <h1 className="m-0 text-[2.4rem] md:text-[clamp(2.8rem,6vw,4.2rem)] leading-none tracking-[-0.05em] font-extrabold">
            Every Ad Platform.
            <br />
            <span className="font-serif italic font-normal text-accent">
              one CLI.
            </span>
          </h1>
          <p className="m-0 max-w-[36rem] text-fg-muted text-[1.05rem] leading-[1.7]">
            Read-only access to Meta, Google, TikTok, and Pinterest through one CLI. Built
            for agents like Claude Code to safely query campaign insights
            across multiple platforms.
          </p>
        </div>

        <div className="flex items-center justify-between max-w-[32rem] py-[0.85rem] px-4 rounded bg-surface-low border-[0.5px] border-outline">
          <code className="text-syn-fg text-[0.9rem]">
            <span className="text-syn-comment mr-2">#</span>
            <span className="text-syn-blue font-bold">npm</span>{" "}
            <span className="text-syn-fg">install</span>{" "}
            <span className="text-syn-cyan">-g</span>{" "}
            <span className="text-syn-orange">agent-ads</span>
          </code>
          <CopyButton copyKey="install" text="npm install -g agent-ads" />
        </div>
      </section>

      {/* ── CLI Examples ────────────────────────────────── */}
      <section className="grid gap-6">
        <SectionHeader
          eyebrow="CLI-First"
          title="Use it directly."
          copy="Run queries straight from your terminal. Every command outputs JSON you can pipe, save, or inspect."
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {homeCLIExamples.map((example) => (
            <article
              key={example.id}
              className="grid gap-3 p-5 rounded bg-surface-low border-[0.5px] border-outline"
            >
              <div className="flex items-center justify-between">
                <span className={eyebrowCls}>{example.engine}</span>
                <span className="text-fg-dim text-[0.75rem]">{example.label}</span>
              </div>
              <CodeBlock
                code={example.command}
                language="bash"
                showHeader={false}
                copyable={false}
              />
            </article>
          ))}
        </div>
      </section>

      {/* ── Agent Integration ───────────────────────────── */}
      <section className="grid gap-6">
        <div className="flex flex-wrap gap-2">
          <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-accent/12 text-accent text-[0.72rem] font-bold tracking-[0.06em] uppercase">
            Agent Integration
          </span>
        </div>
        <SectionHeader
          eyebrow=""
          title="Better with an agent."
          copy="agent-ads is built to work with coding agents like Claude Code. Ask questions in plain English — your agent picks the right command."
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {homePromptCards.map((card) => (
            <article
              key={card.id}
              className="relative grid gap-3 p-5 rounded bg-surface-low border-[0.5px] border-outline overflow-hidden"
            >
              <div className="flex items-center justify-between">
                <span className={eyebrowCls}>{card.category}</span>
                <span
                  className="w-6 h-6 flex items-center justify-center rounded bg-surface-highest/52 text-fg-dim text-[0.72rem]"
                  aria-hidden="true"
                >
                  {card.category.includes("TikTok")
                    ? "\u25B6"
                    : card.category.includes("Pixel")
                      ? "\u2665"
                      : card.category.includes("Google")
                        ? "\u2315"
                        : "\u2913"}
                </span>
              </div>
              <p className="m-0 text-base font-semibold leading-[1.5] text-fg">
                &ldquo;{card.prompt}&rdquo;
              </p>
              <CodeBlock
                code={card.command}
                language="bash"
                showHeader={false}
                copyable={false}
              />
            </article>
          ))}
        </div>
      </section>

      {/* ── Skill for Claude Code ───────────────────────── */}
      <section className="grid gap-6">
        <article
          className="grid gap-4 p-6 rounded bg-surface-low border-[0.5px] border-outline"
        >
          <span className={eyebrowCls}>Claude Code Skill</span>
          <h3 className="m-0 text-[clamp(1.4rem,2.2vw,2rem)] leading-[1.08] tracking-[-0.04em]">
            Install the skill.
          </h3>
          <p className="m-0 text-fg-muted leading-[1.72] max-w-[40rem]">
            agent-ads ships as a skill for Claude Code. Install the CLI, add the
            skill, and your agent can query any supported ad platform on your
            behalf.
          </p>
          <CodeBlock
            code="$ npx skills add https://github.com/bengoism/agent-ads --skill agent-ads"
            language="bash"
            showHeader={false}
            copyable={true}
          />
          <div>
            <Link className={btnSecondary} to="/skills">
              View skill setup &rarr;
            </Link>
          </div>
        </article>
      </section>

      {/* ── Supported Engines ────────────────────────────── */}
      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Supported Platforms"
          title="Supported Platforms"
          copy="Native integration with world-class advertising APIs."
        />

        <div className="grid grid-cols-1 md:grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-3">
          {engineOrder.map((engineId) => {
            const engine = engines[engineId];
            return (
              <article
                key={engineId}
                className="grid gap-3 p-5 rounded bg-surface-low border-[0.5px] border-outline"
              >
                <h3 className="m-0 text-[clamp(1.4rem,2.2vw,2rem)] leading-[1.08] tracking-[-0.04em] flex items-center gap-3">
                  <EngineIcon engineId={engineId} />
                  {engine.name.replace(/ Ads$/, "")}
                </h3>
                <p className="m-0 text-fg-muted leading-[1.72]">
                  {engine.description}
                </p>
                <BadgeList values={engine.tags} />
              </article>
            );
          })}
        </div>
      </section>
    </>
  );
}
