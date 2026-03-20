import { Link } from "react-router";
import {
  engineOrder,
  engines,
  homePromptCards,
  performanceCode,
  performanceFeatures,
  version,
} from "../content/site";
import {
  BadgeList,
  btnPrimary,
  btnSecondary,
  CopyButton,
  eyebrowCls,
  SectionHeader,
  TerminalBlock,
} from "../components/docs";

export const meta = () => [
  { title: "agent-ads | Query Ad Engines from the terminal" },
  {
    name: "description",
    content:
      "A unified, high-performance CLI for Meta, Google, TikTok, and Pinterest ad APIs. Built in Rust for safety and speed.",
  },
];

function highlightCommand(command: string) {
  return command.split("\n").map((line, i) => {
    const parts = line.split(/(this_month|"pix_98765"|campaign\.name|true|"SELECT |campaign\.\.\.")/g);
    return (
      <span key={i}>
        {i > 0 ? "\n" : ""}
        {parts.map((part, j) => {
          if (
            [
              "this_month",
              '"pix_98765"',
              "campaign.name",
              "true",
              '"SELECT ',
              'campaign..."',
            ].includes(part)
          ) {
            return (
              <span key={j} className="text-green">
                {part}
              </span>
            );
          }
          return part;
        })}
      </span>
    );
  });
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
        </div>

        <div className="grid gap-6 max-w-[42rem]">
          <h1 className="m-0 text-[2.4rem] md:text-[clamp(2.8rem,6vw,4.2rem)] leading-none tracking-[-0.05em] font-extrabold">
            Query Ad Engines{" "}
            <span className="font-serif italic font-normal text-accent">from the terminal.</span>
          </h1>
          <p className="m-0 max-w-[36rem] text-fg-muted text-[1.05rem] leading-[1.7]">
            A unified, high-performance interface for Meta, Google, TikTok, and
            Pinterest. Built in <strong className="text-fg font-bold">Rust</strong> for safety and speed.
          </p>
        </div>

        <div className="flex items-center justify-between max-w-[32rem] py-[0.85rem] px-4 rounded bg-surface-low border-[0.5px] border-outline">
          <code className="text-fg-muted text-[0.9rem]">
            <span className="text-fg-dim mr-2">#</span>
            npm install -g agent-ads
          </code>
          <CopyButton copyKey="install" text="npm install -g agent-ads" />
        </div>
      </section>

      {/* ── What You Can Ask ─────────────────────────────── */}
      <section className="grid gap-6">
        <div className="flex flex-wrap gap-2">
          <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-accent/12 text-accent text-[0.72rem] font-bold tracking-[0.06em] uppercase">
            AI-First Interface
          </span>
        </div>
        <SectionHeader
          eyebrow=""
          title="What You Can Ask"
          copy="Transform natural language prompts into powerful CLI queries. Optimized for speed and precision."
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {homePromptCards.map((card) => (
            <article
              key={card.id}
              className="relative grid gap-3 p-5 rounded bg-surface-low border-[0.5px] border-outline overflow-hidden reveal"
              data-reveal
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
              <pre className="m-0 p-3 rounded bg-[rgba(10,10,11,0.95)] border-[0.5px] border-outline text-[0.82rem] leading-[1.6] text-fg-muted overflow-x-auto">
                <code>{highlightCommand(card.command)}</code>
              </pre>
            </article>
          ))}
        </div>
      </section>

      {/* ── Supported Engines ────────────────────────────── */}
      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Supported Engines"
          title="Supported Engines"
          copy="Native integration with world-class advertising APIs."
        />

        <div className="grid grid-cols-1 md:grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-3">
          {engineOrder.map((engineId) => {
            const engine = engines[engineId];
            return (
              <article
                key={engineId}
                className="grid gap-3 p-5 rounded bg-surface-low border-[0.5px] border-outline reveal"
                data-reveal
              >
                <div
                  className="w-9 h-9 flex items-center justify-center rounded bg-surface-highest/62 text-accent text-[0.85rem]"
                  aria-hidden="true"
                >
                  {engineId === "tiktok" ? "\u25B6" : "\u25CF"}
                </div>
                <h3 className="m-0 text-[clamp(1.4rem,2.2vw,2rem)] leading-[1.08] tracking-[-0.04em]">
                  {engine.name}
                </h3>
                <p className="m-0 text-fg-muted leading-[1.72]">{engine.description}</p>
                <BadgeList values={engine.tags} />
              </article>
            );
          })}
        </div>
      </section>

      {/* ── Engineered for Performance ───────────────────── */}
      <section className="grid gap-8">
        <h2 className="m-0 text-[clamp(1.6rem,3vw,2.4rem)] tracking-[-0.04em]">
          Engineered for Performance.
        </h2>

        <div className="grid grid-cols-1 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)] gap-8 items-start">
          <div className="grid gap-6">
            <p className="m-0 text-fg-muted leading-[1.72]">
              Leveraging Rust&apos;s memory safety and concurrency to fetch
              multi-account data up to 10x faster than traditional scripting
              methods.
            </p>

            {performanceFeatures.map((feature) => (
              <div
                key={feature.title}
                className="grid grid-cols-[2.5rem_1fr] gap-3 items-start p-4 rounded bg-surface-low border-[0.5px] border-outline"
              >
                <div
                  className="w-10 h-10 flex items-center justify-center rounded bg-accent/10 text-accent text-[0.85rem]"
                  aria-hidden="true"
                >
                  {feature.title.includes("Binary") ? "\u26A1" : "\u2302"}
                </div>
                <div className="grid gap-[0.2rem]">
                  <strong className="text-[0.95rem]">{feature.title}</strong>
                  <p className="m-0 text-fg-muted leading-[1.72] text-[0.88rem]">
                    {feature.description}
                  </p>
                </div>
              </div>
            ))}
          </div>

          <TerminalBlock
            filename={performanceCode.filename}
            headerComment="# Fetch all active campaigns from Meta"
            lines={performanceCode.lines.map((l) => ({
              num: l.num,
              text: l.text,
              highlight: [...l.highlight],
            }))}
            comment={performanceCode.comment}
          />
        </div>
      </section>

      {/* ── CTA ──────────────────────────────────────────── */}
      <section className="text-center py-12 reveal" data-reveal>
        <h2 className="m-0 mb-6 text-[clamp(1.6rem,3vw,2.4rem)] tracking-[-0.04em]">
          Scale your ad automation.
        </h2>
        <div className="flex flex-col items-stretch gap-3 sm:flex-row sm:items-center sm:justify-center">
          <Link className={btnPrimary} to="/engines/meta">
            Quick Start Guide &rarr;
          </Link>
          <Link className={btnSecondary} to="/skills">
            Explore Examples
          </Link>
        </div>
      </section>
    </>
  );
}
