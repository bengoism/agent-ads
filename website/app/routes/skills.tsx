import { Link } from "react-router";
import { generatedContent } from "../generated/content";
import { engineOrder, skillInstallCommand, skillStartCommands } from "../content/site";
import { CodeBlock } from "../components/code-block";
import {
  btnPrimary,
  btnSecondary,
  CommandPanel,
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
        <SectionHeader
          eyebrow="Skill"
          title="Install the skill"
          copy="Useful for Codex, Claude Code, and similar agents because it keeps provider routing explicit, points the agent to the right provider docs, and reduces made-up cross-provider commands."
        />

        <div className="flex flex-col items-stretch gap-3 sm:flex-row sm:items-center">
          <Link className={btnPrimary} to="/reference">
            Browse providers
          </Link>
          <Link className={btnSecondary} to="/auth">
            Browse auth
          </Link>
        </div>

        <div className="grid gap-4 lg:grid-cols-[minmax(0,1.1fr)_minmax(18rem,0.9fr)]">
          <CommandPanel
            eyebrow="Install"
            title="Install the public skill"
            command={skillInstallCommand}
            copyKey="skills-install"
          />

          <article className="grid gap-4 p-4 rounded bg-gradient-to-b from-[rgba(32,31,32,0.96)] to-[rgba(16,16,17,0.98)] border-[0.5px] border-outline shadow-ambient">
            <span className={eyebrowCls}>Why it helps</span>
            <h2 className="m-0 text-[1.2rem] leading-[1.15] tracking-[-0.03em]">
              Useful for agents
            </h2>
            <p className="m-0 text-fg-muted leading-[1.72]">
              The skill gives agents a tighter path through the CLI: start with available
              providers, confirm auth state, then route into the provider guide that matches the
              task instead of inventing a generic ad-platform command surface.
            </p>
            <ul className="m-0 pl-5 grid gap-2 text-fg-muted leading-[1.62]">
              <li>Keep commands explicit with <code className="markdown__inline-code">agent-ads &lt;provider&gt; ...</code></li>
              <li>Route Meta, Google, TikTok, Pinterest, LinkedIn, and X work into the right docs</li>
              <li>Reduce wrong flags, wrong auth models, and cross-provider guesswork</li>
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
          </article>
        </div>
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
