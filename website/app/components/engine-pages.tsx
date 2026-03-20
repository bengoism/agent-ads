import { Link } from "react-router";
import { generatedContent } from "../generated/content";
import { engines, type EngineId } from "../content/site";
import {
  BadgeList,
  btnPrimary,
  btnSecondary,
  CommandPanel,
  eyebrowCls,
  MarkdownBlock,
  MetricGrid,
  PageHero,
  RouteCard,
  SectionHeader,
} from "./docs";

function buildEngineReferenceHrefResolver(engineId: EngineId) {
  const docs = generatedContent.references[engineId];
  const docRoutes = new Map<string, string>(
    docs.map((doc) => [doc.file, `/reference/${engineId}#${doc.id}`]),
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

export function EngineQuickStartPage({ engineId }: { engineId: EngineId }) {
  const engine = engines[engineId];
  const quickStart = generatedContent.quickStarts[engineId];
  const referenceDocs = generatedContent.references[engineId];
  const docCount = referenceDocs.length;

  return (
    <>
      <div id="engine-overview">
        <PageHero
          eyebrow={`${engine.name} quick start`}
          title={
            <>
              Start with {engine.name}
              <span className="text-fg/78"> in a few working commands.</span>
            </>
          }
          lede={engine.quickStartLead}
          detail={`${quickStart.steps.length} setup steps`}
          actions={
            <>
              <Link className={btnPrimary} to={`/reference/${engineId}`}>
                Open {engine.name} reference
              </Link>
              <a className={btnSecondary} href="#reference-modules">
                Jump to guides
              </a>
            </>
          }
          aside={
            <div className="grid gap-[1.2rem]">
              <CommandPanel
                eyebrow="First command"
                title={engine.eyebrow}
                command={engine.firstCommand}
                copyKey={`${engineId}-first-command`}
              />
              <MetricGrid metrics={engine.stats} />
            </div>
          }
        />
      </div>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Quick start"
          title={`${engine.name} setup`}
          copy={quickStart.intro || `Use this page when you need the shortest credible setup path for ${engine.name}.`}
        />

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
                className="grid grid-cols-1 sm:grid-cols-[auto_minmax(0,1fr)] gap-4 p-5 rounded bg-gradient-to-b from-[rgba(28,27,28,0.92)] to-[rgba(18,18,19,0.98)] shadow-ambient reveal"
                data-reveal
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

      <section id="reference-modules" className="grid gap-6">
        <SectionHeader
          eyebrow="Reference"
          title={`${engine.name} guides`}
          copy={`Use these ${docCount} guides when you need exact flags, auth details, or deeper workflow notes for ${engine.name}.`}
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {referenceDocs.map((doc) => (
            <RouteCard
              key={doc.id}
              eyebrow={engine.eyebrow}
              title={doc.title}
              copy={doc.summary}
              to={`/reference/${engineId}#${doc.id}`}
              cta="Open guide"
            />
          ))}
        </div>
      </section>
    </>
  );
}

export function EngineReferencePage({ engineId }: { engineId: EngineId }) {
  const engine = engines[engineId];
  const docs = generatedContent.references[engineId];
  const resolveHref = buildEngineReferenceHrefResolver(engineId);

  return (
    <>
      <PageHero
        eyebrow={`${engine.name} reference`}
        title={
          <>
            {engine.name} command reference
            <span className="text-fg/78"> and workflow docs.</span>
          </>
        }
        lede={engine.referenceLead}
        detail={`${docs.length} guides`}
        actions={
          <>
            <Link className={btnPrimary} to={`/engines/${engineId}`}>
              Open quick start
            </Link>
            <a className={btnSecondary} href={`#${docs[0]?.id ?? ""}`}>
              Jump to docs
            </a>
          </>
        }
        aside={
          <div className="grid gap-[1.2rem]">
            <CommandPanel
              eyebrow="First command"
              title={engine.eyebrow}
              command={engine.firstCommand}
              copyKey={`${engineId}-reference-command`}
            />
            <BadgeList values={engine.tags} />
          </div>
        }
      />

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Reference"
          title={`${engine.name} guides`}
          copy="Use these guides for exact flags, auth requirements, and deeper workflow recipes."
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-[0.9rem]">
          {docs.map((doc) => (
            <a
              key={doc.id}
              className="grid gap-[0.45rem] p-4 rounded bg-surface-low border-[0.5px] border-outline text-fg-muted shadow-ambient hover:bg-surface-highest/42 hover:text-fg"
              href={`#${doc.id}`}
            >
              <span className={eyebrowCls}>Guide</span>
              <strong className="text-fg">{doc.title}</strong>
              <span className="leading-[1.65]">{doc.summary}</span>
            </a>
          ))}
        </div>
      </section>

      <section className="grid gap-6">
        {docs.map((doc) => (
          <article
            key={doc.id}
            id={doc.id}
            className="grid gap-5 relative overflow-hidden p-[1.2rem] rounded bg-gradient-to-b from-[rgba(32,31,32,0.96)] to-[rgba(16,16,17,0.98)] shadow-ambient reveal"
            data-reveal
          >
            <div className="grid grid-cols-1 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,0.85fr)] gap-4">
              <div>
                <span className={eyebrowCls}>Guide</span>
                <h2 className="m-0 text-[clamp(1.4rem,2.2vw,2rem)] leading-[1.08] tracking-[-0.04em]">
                  {doc.title}
                </h2>
                <p className="m-0 text-fg-muted leading-[1.72]">{doc.summary}</p>
              </div>
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
