import { Link } from "react-router";
import { generatedContent } from "../generated/content";
import { homePromptCards } from "../content/site";
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

  if (["meta", "google", "tiktok", "pinterest"].includes(engineId)) {
    return `/reference/${engineId}`;
  }

  return href;
}

export const meta = () => [
  { title: "Skills | agent-ads" },
  {
    name: "description",
    content:
      "Agent integration guidance for agent-ads: prompts, glossary, command rules, engine routing, and operational constraints.",
  },
];

export default function SkillsRoute() {
  const [whatYouCanAsk, ...remainingSections] = generatedContent.skills.sections;
  const featuredWorkflow = homePromptCards[0];

  return (
    <>
      <PageHero
        eyebrow="Agent integration"
        title={
          <>
            Translate prompts
            <span className="text-fg/78"> into explicit engine commands.</span>
          </>
        }
        lede={generatedContent.skills.description}
        detail="Prompt routing guide"
        actions={
          <>
            <Link className={btnPrimary} to="/engines/meta">
              Browse engines
            </Link>
            <Link className={btnSecondary} to="/reference">
              Browse reference docs
            </Link>
          </>
        }
        aside={
          <CommandPanel
            eyebrow="Example workflow"
            title={featuredWorkflow.prompt}
            command={featuredWorkflow.command}
            copyKey="skills-translation"
          />
        }
      />

      <section id={whatYouCanAsk.id} className="grid gap-6">
        <SectionHeader
          eyebrow="Examples"
          title="What agents can ask"
          copy={whatYouCanAsk.summary}
        />

        <article
          className="grid gap-5 relative overflow-hidden p-[1.2rem] rounded bg-gradient-to-b from-[rgba(32,31,32,0.96)] to-[rgba(16,16,17,0.98)] shadow-ambient reveal"
          data-reveal
        >
          <MarkdownBlock markdown={whatYouCanAsk.markdown} resolveHref={resolveSkillHref} />
        </article>
      </section>

      {remainingSections.map((section) => (
        <section key={section.id} id={section.id} className="grid gap-6">
          <SectionHeader eyebrow="Guide" title={section.title} copy={section.summary} />
          <article
            className="grid gap-5 relative overflow-hidden p-[1.2rem] rounded bg-gradient-to-b from-[rgba(32,31,32,0.96)] to-[rgba(16,16,17,0.98)] shadow-ambient reveal"
            data-reveal
          >
            <MarkdownBlock markdown={section.markdown} resolveHref={resolveSkillHref} />
          </article>
        </section>
      ))}
    </>
  );
}
