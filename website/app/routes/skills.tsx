import { generatedContent } from "../generated/content";
import { engineOrder, skillInstallCommand } from "../content/site";
import {
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
      "Install the agent-ads skill so Codex and Claude Code know how to inspect providers, check auth, and use agent-ads commands.",
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
          copy="Install the public skill if you use Codex or Claude Code with agent-ads. It gives the agent a simple playbook for checking providers, checking auth, and using the real commands."
        />

        <div className="grid gap-4 max-w-4xl">
          <CommandPanel
            eyebrow="Install"
            title="Add the public skill"
            command={skillInstallCommand}
            copyKey="skills-install"
          />

          <article className="grid gap-4 p-4 rounded bg-surface-low border-[0.5px] border-outline shadow-ambient">
            <div className="grid gap-2">
              <span className={eyebrowCls}>Why use it</span>
              <h3 className="m-0 text-[1.05rem] leading-[1.35]">Useful with Codex and Claude Code</h3>
              <p className="m-0 text-fg-muted leading-[1.72]">
                The skill helps the agent understand how to use <code className="markdown__inline-code">agent-ads</code> without guessing. It points the agent to the right provider docs and keeps commands provider-specific.
              </p>
            </div>
            <ul className="m-0 pl-5 grid gap-2 text-fg-muted leading-[1.62]">
              <li>Start by listing providers and checking auth status</li>
              <li>Use commands like <code className="markdown__inline-code">agent-ads meta ...</code> or <code className="markdown__inline-code">agent-ads tiktok ...</code></li>
              <li>Follow the provider guides instead of inventing shared ads commands</li>
            </ul>
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
