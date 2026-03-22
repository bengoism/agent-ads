import { useMemo } from "react";
import { generatedContent } from "../generated/content";
import { engines, type EngineId } from "../content/site";
import {
  CommandPanel,
  eyebrowCls,
  MarkdownBlock,
  SectionHeader,
} from "./docs";
import { useToc, type TocItem } from "./toc-context";

function buildEngineReferenceHrefResolver(engineId: EngineId) {
  const docs = generatedContent.references[engineId];
  const docRoutes = new Map<string, string>(
    docs.map((doc) => [doc.file, `#${doc.id}`]),
  );

  return (href: string) => {
    if (
      !href ||
      href.startsWith("#") ||
      href.startsWith("http://") ||
      href.startsWith("https://") ||
      href.startsWith("mailto:")
    ) {
      return href;
    }

    const [pathPart] = href.split("#");
    const fileName = pathPart?.split("/").pop();

    if (!fileName) {
      return href;
    }

    return docRoutes.get(fileName) ?? href;
  };
}

function removeFirstBashBlock(markdown: string) {
  return markdown.replace(/```bash[\s\S]*?```/, "").trim();
}

function removeDocumentHeading(markdown: string) {
  return markdown.replace(/^#\s+.+\n+/, "").trim();
}

function removeLeadingSummary(markdown: string, summary: string) {
  if (!summary) {
    return markdown;
  }

  const blocks = markdown.split(/\n{2,}/);

  if (!blocks.length) {
    return markdown;
  }

  const firstBlock = blocks[0]?.replace(/\s+/g, " ").trim();
  const normalizedSummary = summary.replace(/\s+/g, " ").trim();

  if (firstBlock === normalizedSummary) {
    blocks.shift();
  }

  return blocks.join("\n\n").trim();
}

export function EnginePage({ engineId }: { engineId: EngineId }) {
  const engine = engines[engineId];
  const quickStart = generatedContent.quickStarts[engineId];
  const referenceDocs = generatedContent.references[engineId];
  const resolveHref = buildEngineReferenceHrefResolver(engineId);

  const tocItems = useMemo<TocItem[]>(
    () => [
      ...quickStart.steps.map((s) => ({ id: s.id, label: `${s.stepNumber}. ${s.title}` })),
      ...referenceDocs.map((d) => ({ id: d.id, label: d.title })),
    ],
    [quickStart.steps, referenceDocs],
  );
  useToc(tocItems);

  return (
    <>
      <div className="grid gap-4">
        <h1 className="m-0 text-[clamp(1.6rem,3vw,2.4rem)] leading-[1.08] tracking-[-0.04em]">
          {engine.name}
        </h1>
        <ol className="m-0 p-0 list-none grid gap-1 lg:hidden">
          {quickStart.steps.map((step) => (
            <li key={step.id}>
              <a
                className="text-fg-muted hover:text-accent text-[0.88rem] leading-[1.6]"
                href={`#${step.id}`}
              >
                {step.stepNumber}. {step.title}
              </a>
            </li>
          ))}
          {referenceDocs.map((doc) => (
            <li key={doc.id}>
              <a
                className="text-fg-muted hover:text-accent text-[0.88rem] leading-[1.6]"
                href={`#${doc.id}`}
              >
                {doc.title}
              </a>
            </li>
          ))}
        </ol>
      </div>

      {/* ── Quick start steps ──────────────────────────────── */}
      <section className="grid gap-6">
        <div className="grid gap-4">
          {quickStart.steps.map((step) => {
            const supportingMarkdown = removeLeadingSummary(
              removeFirstBashBlock(step.markdown),
              step.summary,
            );

            return (
              <article
                key={step.id}
                id={step.id}
                className="grid grid-cols-1 sm:grid-cols-[auto_minmax(0,1fr)] gap-4 p-5 rounded bg-gradient-to-b from-[rgba(28,27,28,0.92)] to-[rgba(18,18,19,0.98)] shadow-ambient"
              >
                <div className="flex items-start">
                  <span className="inline-flex items-center justify-center w-[3.25rem] h-[3.25rem] rounded bg-surface-highest/70 border-[0.5px] border-outline text-accent font-mono text-base font-bold">
                    {String(step.stepNumber).padStart(2, "0")}
                  </span>
                </div>
                <div className="grid gap-[0.9rem]">
                  <span className={eyebrowCls}>Step {String(step.stepNumber).padStart(2, "0")}</span>
                  <h2 className="m-0 text-[clamp(1.4rem,2.2vw,2rem)] leading-[1.08] tracking-[-0.04em]">
                    {step.title}
                  </h2>
                  <p className="m-0 text-fg-muted leading-[1.72]">{step.summary}</p>
                  {step.primaryCommand ? (
                    <CommandPanel
                      compact
                      eyebrow="Primary command"
                      title={step.title}
                      command={step.primaryCommand}
                      copyKey={`${engineId}-${step.id}`}
                    />
                  ) : null}
                  {supportingMarkdown ? <MarkdownBlock markdown={supportingMarkdown} /> : null}
                </div>
              </article>
            );
          })}
        </div>
      </section>

      {/* ── Reference guides ───────────────────────────────── */}
      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Reference"
          title={`${engine.name} guides`}
          copy={`${referenceDocs.length} guides — exact flags, auth details, and workflow notes.`}
        />

        {referenceDocs.map((doc) => (
          <article
            key={doc.id}
            id={doc.id}
            className="grid gap-5 relative overflow-hidden p-[1.2rem] rounded bg-gradient-to-b from-[rgba(32,31,32,0.96)] to-[rgba(16,16,17,0.98)] shadow-ambient"
          >
            <div className="grid gap-2">
              <span className={eyebrowCls}>Guide</span>
              <h2 className="m-0 text-[clamp(1.4rem,2.2vw,2rem)] leading-[1.08] tracking-[-0.04em]">
                {doc.title}
              </h2>
              <p className="m-0 text-fg-muted leading-[1.72]">{doc.summary}</p>
              {doc.primaryCommand ? (
                <CommandPanel
                  compact
                  eyebrow="First command"
                  title={doc.title}
                  command={doc.primaryCommand}
                  copyKey={`${engineId}-${doc.id}`}
                />
              ) : null}
            </div>
            <MarkdownBlock
              markdown={removeLeadingSummary(removeDocumentHeading(doc.markdown), doc.summary)}
              resolveHref={resolveHref}
            />
          </article>
        ))}
      </section>
    </>
  );
}
