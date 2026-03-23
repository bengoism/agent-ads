import { Link } from "react-router";
import { generatedContent } from "../generated/content";
import { engineOrder, skillStartCommands } from "../content/site";
import {
  btnPrimary,
  btnSecondary,
  CommandPanel,
  MarkdownBlock,
  PageHero,
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
      "Operational skill guide for agent-ads: common tasks, command rules, provider routing, and shared CLI behavior.",
  },
];

export default function SkillsRoute() {
  const [commonTasks, ...remainingSections] = generatedContent.skills.sections;

  return (
    <>
      <PageHero
        eyebrow="Skill"
        title={
          <>
            Keep provider
            <span className="text-fg/78"> routing explicit.</span>
          </>
        }
        lede={generatedContent.skills.description}
        detail="Operational guide"
        actions={
          <>
            <Link className={btnPrimary} to="/reference">
              Browse providers
            </Link>
            <Link className={btnSecondary} to="/auth">
              Browse auth
            </Link>
          </>
        }
        aside={
          <CommandPanel
            eyebrow="Start here"
            title="Inspect providers and auth state"
            command={skillStartCommands}
            copyKey="skills-start"
          />
        }
      />

      <section id={commonTasks.id} className="grid gap-6">
        <SectionHeader
          eyebrow="Tasks"
          title="Common tasks"
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
