import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const websiteRoot = path.resolve(__dirname, "..");
const repoRoot = path.resolve(websiteRoot, "..");
const outputDir = path.join(websiteRoot, "app", "generated");
const outputFile = path.join(outputDir, "content.ts");

const providerReferenceFiles = {
  meta: [
    "meta.md",
    "meta-auth-and-output.md",
    "meta-accounts-and-objects.md",
    "meta-reports.md",
    "meta-creative-and-changes.md",
    "meta-tracking.md",
    "meta-workflows.md",
  ],
  google: ["google.md", "google-workflows.md"],
  tiktok: [
    "tiktok.md",
    "tiktok-auth.md",
    "tiktok-accounts-and-objects.md",
    "tiktok-reports.md",
    "tiktok-creative-and-tracking.md",
    "tiktok-workflows.md",
  ],
  pinterest: ["pinterest.md"],
  linkedin: ["linkedin.md"],
  x: [
    "x.md",
    "x-auth-and-config.md",
    "x-campaign-management.md",
    "x-creatives.md",
    "x-audiences-and-measurement.md",
    "x-analytics.md",
  ],
};

function slugify(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function parseSections(markdown, level) {
  const prefix = "#".repeat(level);
  const expression = new RegExp(`^${prefix}\\s+(.+)$`, "gm");
  const matches = [...markdown.matchAll(expression)];

  return matches.map((match, index) => {
    const heading = match[1].trim();
    const start = (match.index ?? 0) + match[0].length;
    const end = index + 1 < matches.length ? matches[index + 1].index ?? markdown.length : markdown.length;

    return {
      heading,
      body: markdown.slice(start, end).trim(),
    };
  });
}

function stripFrontmatter(markdown) {
  if (!markdown.startsWith("---")) {
    return markdown;
  }

  const end = markdown.indexOf("\n---", 3);

  if (end === -1) {
    return markdown;
  }

  return markdown.slice(end + 4).trimStart();
}

function getMarkdownSection(markdown, heading) {
  const expression = new RegExp(`^##\\s+${heading.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\s*$`, "m");
  const match = expression.exec(markdown);

  if (!match || match.index == null) {
    throw new Error(`Missing section: ${heading}`);
  }

  const start = match.index + match[0].length;
  const nextHeading = markdown.slice(start).search(/^##\s+/m);
  const end = nextHeading === -1 ? markdown.length : start + nextHeading;

  return markdown.slice(start, end).trim();
}

function stripMarkdown(markdown) {
  return markdown
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/^\|.*$/gm, " ")
    .replace(/^[-*+]\s+/gm, "")
    .replace(/^\d+\.\s+/gm, "")
    .replace(/^>\s+/gm, "")
    .replace(/[*_#]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function firstParagraph(markdown) {
  const proseOnly = markdown
    .replace(/```[\s\S]*?```/g, "")
    .replace(/^\|.*$/gm, "")
    .replace(/\r\n/g, "\n");

  const blocks = proseOnly
    .replace(/\r\n/g, "\n")
    .split(/\n{2,}/)
    .map((block) => block.trim())
    .filter(Boolean);

  for (const block of blocks) {
    if (
      block.startsWith("#") ||
      block.startsWith("- ") ||
      block.startsWith("* ") ||
      /^\d+\./.test(block)
    ) {
      continue;
    }

    const plain = stripMarkdown(block);

    if (plain) {
      return plain;
    }
  }

  return "";
}

function firstCommand(markdown) {
  const bashBlock = markdown.match(/```bash\s+([\s\S]*?)```/);

  if (!bashBlock) {
    return "";
  }

  return bashBlock[1].trim();
}

async function buildQuickStarts() {
  const readme = await readFile(path.join(repoRoot, "README.md"), "utf8");

  return {
    meta: parseQuickStart(readme, "Meta"),
    google: parseQuickStart(readme, "Google"),
    tiktok: parseQuickStart(readme, "TikTok"),
    pinterest: parseQuickStart(readme, "Pinterest"),
    linkedin: parseQuickStart(readme, "LinkedIn"),
    x: parseQuickStart(readme, "X"),
  };
}

function parseQuickStart(readme, providerName) {
  const section = getMarkdownSection(readme, `Quick Start (${providerName})`);
  const stepSections = parseSections(section, 3);
  const intro = firstParagraph(stepSections.length ? section.slice(0, section.indexOf(`### ${stepSections[0].heading}`)) : section);
  const steps = stepSections.map((step) => {
    const titleMatch = step.heading.match(/^(\d+)\.\s+(.+)$/);
    const stepNumber = titleMatch ? Number(titleMatch[1]) : 0;
    const title = titleMatch ? titleMatch[2] : step.heading;
    const id = `step-${slugify(title)}`;
    const summary = firstParagraph(step.body);

    return {
      id,
      stepNumber,
      title,
      summary,
      primaryCommand: firstCommand(step.body),
      markdown: step.body,
    };
  });

  return {
    intro,
    steps,
  };
}

async function buildSkillsContent() {
  const skillMarkdown = await readFile(path.join(repoRoot, "skills", "agent-ads", "SKILL.md"), "utf8");
  const skillBody = stripFrontmatter(skillMarkdown);
  const preamble = skillBody.split(/^##\s+/m)[0];
  const description = firstParagraph(preamble);

  const sectionNames = [
    "Common Tasks",
    "Command Syntax Rules",
    "Provider Routing",
    "Shared Behavior",
    "Common Issues",
    "Stop Conditions",
  ];

  return {
    description,
    sections: sectionNames.map((heading) => {
      const markdown = getMarkdownSection(skillBody, heading);

      return {
        id: slugify(heading),
        title: heading,
        summary: firstParagraph(markdown),
        markdown,
      };
    }),
  };
}

async function buildReferenceContent() {
  const referencesRoot = path.join(repoRoot, "skills", "agent-ads", "references");
  const providers = {};

  for (const [providerId, files] of Object.entries(providerReferenceFiles)) {
    const docs = [];

    for (const file of files) {
      const markdown = await readFile(path.join(referencesRoot, file), "utf8");
      const title = markdown.match(/^#\s+(.+)$/m)?.[1].trim() ?? file.replace(/\.md$/, "");

      docs.push({
        id: slugify(title),
        title,
        file,
        summary: firstParagraph(markdown.replace(/^#\s+.+$/m, "").trim()),
        primaryCommand: firstCommand(markdown),
        markdown,
      });
    }

    providers[providerId] = docs;
  }

  return providers;
}

async function main() {
  const generatedContent = {
    generatedAt: new Date().toISOString(),
    quickStarts: await buildQuickStarts(),
    skills: await buildSkillsContent(),
    references: await buildReferenceContent(),
  };

  await mkdir(outputDir, { recursive: true });
  await writeFile(
    outputFile,
    [
      "// This file is generated by scripts/generate-content.mjs.",
      "// Do not edit it by hand.",
      "",
      `export const generatedContent = ${JSON.stringify(generatedContent, null, 2)} as const;`,
      "",
      "export type GeneratedContent = typeof generatedContent;",
      "",
    ].join("\n"),
  );
}

await main();
