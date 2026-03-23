import { Link } from "react-router";
import { generatedContent } from "../generated/content";
import { engineOrder, skillInstallCommand, skillStartCommands } from "../content/site";
import { CodeBlock } from "../components/code-block";
import {
  btnPrimary,
  btnSecondary,
  eyebrowCls,
  MarkdownBlock,
  SectionHeader,
} from "../components/docs";

function resolveSkillHref(href: string) {
  if (
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

  const engineId = fileName.replace(/\.md$/, "");

  if ((engineOrder as readonly string[]).includes(engineId)) {
    return `/engines/${engineId}`;
  }

  return href;
}

export const meta = () => [
  { title: "Skill | agent-ads" },
  {
    name: "description",
    content:
      "Install and use the agent-ads skill with Codex, Claude Code, and similar agents to keep provider routing explicit.",
  },
];

export default function SkillsRoute() {
  const [commonTasks, ...remainingSections] = generatedContent.skills.sections;

  return (
    <>
      <section id="skill-install" className="grid gap-6">
        <article className="grid gap-6 p-[1.2rem] rounded bg-gradient-to-b from-[rgba(28,27,28,0.96)] to-[rgba(16,16,17,0.98)] border-[0.5px] border-outline shadow-ambient lg:grid-cols-[minmax(0,1.1fr)_minmax(18rem,0.9fr)]">
          <div className="grid gap-4">
            <span className={eyebrowCls}>Skill</span>
            <h1 className="m-0 text-[clamp(1.9rem,3vw,2.8rem)] leading-[0.98] tracking-[-0.05em]">
              Useful for agents
            </h1>
            <p className="m-0 text-fg-muted leading-[1.72] max-w-[34rem]">
              Use the public skill with Codex, Claude Code, and similar agents when you want the
              agent to stay inside the real provider-first CLI instead of inventing a generic ads
              abstraction.
            </p>
            <div className="flex flex-wrap gap-2">
              <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-accent/12 text-accent text-[0.72rem] font-bold tracking-[0.06em] uppercase">
                Provider-first
              </span>
              <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-surface-highest/52 text-fg-muted text-[0.72rem] font-bold tracking-[0.06em] uppercase">
                Explicit commands
              </span>
              <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-surface-highest/52 text-fg-muted text-[0.72rem] font-bold tracking-[0.06em] uppercase">
                Provider docs
              </span>
            </div>
            <div className="grid gap-2">
              <span className={eyebrowCls}>Install</span>
              <CodeBlock
                code={skillInstallCommand}
                language="bash"
                showHeader={false}
                copyable={true}
              />
            </div>
          </div>

          <div className="grid gap-4 p-4 rounded bg-[rgba(14,14,15,0.72)] border-[0.5px] border-outline">
            <span className={eyebrowCls}>How agents use it</span>
            <p className="m-0 text-fg-muted leading-[1.72]">
              The skill narrows the search space for the agent: inspect providers, confirm auth,
              then jump to the right provider guide before composing a command.
            </p>
            <ul className="m-0 pl-5 grid gap-2 text-fg-muted leading-[1.62]">
              <li>
                Keep commands explicit with{" "}
                <code className="markdown__inline-code">agent-ads &lt;provider&gt; ...</code>
              </li>
              <li>Route Meta, Google, TikTok, Pinterest, LinkedIn, and X work into the right docs</li>
              <li>Reduce wrong flags, auth mixups, and cross-provider guesswork</li>
            </ul>
            <div className="grid gap-2">
              <span className={eyebrowCls}>Start here</span>
              <CodeBlock
                code={skillStartCommands}
                language="bash"
                showHeader={false}
                copyable={true}
              />
            </div>
            <div className="flex flex-col items-stretch gap-3 sm:flex-row sm:items-center">
              <Link className={btnPrimary} to="/reference">
                Browse providers
              </Link>
              <Link className={btnSecondary} to="/auth">
                Browse auth
              </Link>
            </div>
          </div>
        </article>
      </section>

      <section id={commonTasks.id} className="grid gap-6">
        <SectionHeader
          eyebrow="Use Cases"
          title="What agents use it for"
          copy={commonTasks.summary}
        />

        <article
          className="grid gap-5 relative overflow-hidden p-[1.2rem] rounded bg-gradient-to-b from-[rgba(32,31,32,0.96)] to-[rgba(16,16,17,0.98)] shadow-ambient"
        >
          <MarkdownBlock markdown={commonTasks.markdown} resolveHref={resolveSkillHref} />
        </article>
      </section>

      {remainingSections.map((section) => (
        <section key={section.id} id={section.id} className="grid gap-6">
          <SectionHeader eyebrow="Guide" title={section.title} copy={section.summary} />
          <article
            className="grid gap-5 relative overflow-hidden p-[1.2rem] rounded bg-gradient-to-b from-[rgba(32,31,32,0.96)] to-[rgba(16,16,17,0.98)] shadow-ambient"
          >
            <MarkdownBlock markdown={section.markdown} resolveHref={resolveSkillHref} />
          </article>
        </section>
      ))}
    </>
  );
}
