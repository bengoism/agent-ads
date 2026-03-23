#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

type Section = {
  title: string;
  intro?: string;
  args: string[];
  language?: string;
};

const root = path.resolve(__dirname, "..", "..", "..");
const outputPath = path.join(root, "docs", "command-topics.md");

const sections: Section[] = [
  {
    title: "Root Help",
    intro:
      "Canonical syntax uses space-separated subcommands. Colon-delimited forms are intentionally undocumented and unsupported.",
    args: ["--help"],
    language: "text"
  },
  {
    title: "Providers List",
    intro: "Inspect the currently available and planned provider namespaces.",
    args: ["providers", "list"],
    language: "json"
  },
  {
    title: "Root Auth",
    intro: "Inspect aggregated auth status or launch guided local setup or credential clearing.",
    args: ["auth", "--help"],
    language: "text"
  },
  {
    title: "Meta Topic",
    intro: "The Meta provider owns all currently implemented ad commands.",
    args: ["meta", "--help"],
    language: "text"
  },
  {
    title: "Meta Businesses",
    args: ["meta", "businesses", "--help"],
    language: "text"
  },
  {
    title: "Meta Ad Accounts",
    args: ["meta", "ad-accounts", "--help"],
    language: "text"
  },
  {
    title: "Meta Campaigns",
    args: ["meta", "campaigns", "--help"],
    language: "text"
  },
  {
    title: "Meta Insights",
    args: ["meta", "insights", "--help"],
    language: "text"
  },
  {
    title: "Meta Report Runs",
    args: ["meta", "report-runs", "--help"],
    language: "text"
  },
  {
    title: "Meta Creatives",
    args: ["meta", "creatives", "--help"],
    language: "text"
  },
  {
    title: "Meta Activities",
    args: ["meta", "activities", "--help"],
    language: "text"
  },
  {
    title: "Meta Tracking",
    intro:
      "Tracking and measurement-health commands stay provider-specific; there is no shared cross-provider analytics abstraction.",
    args: ["meta", "pixel-health", "--help"],
    language: "text"
  },
  {
    title: "Meta Auth",
    args: ["meta", "auth", "--help"],
    language: "text"
  },
  {
    title: "Meta Config",
    args: ["meta", "config", "--help"],
    language: "text"
  },
  {
    title: "TikTok Topic",
    intro: "The TikTok provider covers the TikTok Business API.",
    args: ["tiktok", "--help"],
    language: "text"
  },
  {
    title: "TikTok Advertisers",
    args: ["tiktok", "advertisers", "--help"],
    language: "text"
  },
  {
    title: "TikTok Campaigns",
    args: ["tiktok", "campaigns", "--help"],
    language: "text"
  },
  {
    title: "TikTok Insights",
    args: ["tiktok", "insights", "--help"],
    language: "text"
  },
  {
    title: "TikTok Report Runs",
    args: ["tiktok", "report-runs", "--help"],
    language: "text"
  },
  {
    title: "TikTok Creatives",
    args: ["tiktok", "creatives", "--help"],
    language: "text"
  },
  {
    title: "TikTok Auth",
    args: ["tiktok", "auth", "--help"],
    language: "text"
  },
  {
    title: "TikTok Config",
    args: ["tiktok", "config", "--help"],
    language: "text"
  },
  {
    title: "Pinterest Topic",
    intro: "The Pinterest provider covers the Pinterest Ads API.",
    args: ["pinterest", "--help"],
    language: "text"
  },
  {
    title: "Pinterest Ad Accounts",
    args: ["pinterest", "ad-accounts", "--help"],
    language: "text"
  },
  {
    title: "Pinterest Campaigns",
    args: ["pinterest", "campaigns", "--help"],
    language: "text"
  },
  {
    title: "Pinterest Analytics",
    args: ["pinterest", "analytics", "--help"],
    language: "text"
  },
  {
    title: "Pinterest Report Runs",
    args: ["pinterest", "report-runs", "--help"],
    language: "text"
  },
  {
    title: "Pinterest Audiences",
    args: ["pinterest", "audiences", "--help"],
    language: "text"
  },
  {
    title: "Pinterest Auth",
    args: ["pinterest", "auth", "--help"],
    language: "text"
  },
  {
    title: "Pinterest Config",
    args: ["pinterest", "config", "--help"],
    language: "text"
  },
  {
    title: "LinkedIn Topic",
    intro: "The LinkedIn provider covers the LinkedIn Marketing API.",
    args: ["linkedin", "--help"],
    language: "text"
  },
  {
    title: "LinkedIn Ad Accounts",
    args: ["linkedin", "ad-accounts", "--help"],
    language: "text"
  },
  {
    title: "LinkedIn Campaign Groups",
    args: ["linkedin", "campaign-groups", "--help"],
    language: "text"
  },
  {
    title: "LinkedIn Campaigns",
    args: ["linkedin", "campaigns", "--help"],
    language: "text"
  },
  {
    title: "LinkedIn Creatives",
    args: ["linkedin", "creatives", "--help"],
    language: "text"
  },
  {
    title: "LinkedIn Analytics",
    args: ["linkedin", "analytics", "--help"],
    language: "text"
  },
  {
    title: "LinkedIn Auth",
    args: ["linkedin", "auth", "--help"],
    language: "text"
  },
  {
    title: "LinkedIn Config",
    args: ["linkedin", "config", "--help"],
    language: "text"
  },
  {
    title: "X Topic",
    intro: "The X provider covers the read-only X Ads API surface.",
    args: ["x", "--help"],
    language: "text"
  },
  {
    title: "X Accounts",
    args: ["x", "accounts", "--help"],
    language: "text"
  },
  {
    title: "X Campaigns",
    args: ["x", "campaigns", "--help"],
    language: "text"
  },
  {
    title: "X Line Items",
    args: ["x", "line-items", "--help"],
    language: "text"
  },
  {
    title: "X Promoted Tweets",
    args: ["x", "promoted-tweets", "--help"],
    language: "text"
  },
  {
    title: "X Analytics",
    args: ["x", "analytics", "--help"],
    language: "text"
  },
  {
    title: "X Auth",
    args: ["x", "auth", "--help"],
    language: "text"
  },
  {
    title: "X Config",
    args: ["x", "config", "--help"],
    language: "text"
  },
  {
    title: "Google Topic",
    intro: "The Google provider covers read-only Google Ads and native GAQL access.",
    args: ["google", "--help"],
    language: "text"
  },
  {
    title: "Google Customers",
    args: ["google", "customers", "--help"],
    language: "text"
  },
  {
    title: "Google Campaigns",
    args: ["google", "campaigns", "--help"],
    language: "text"
  },
  {
    title: "Google GAQL",
    args: ["google", "gaql", "--help"],
    language: "text"
  },
  {
    title: "Google Auth",
    args: ["google", "auth", "--help"],
    language: "text"
  },
  {
    title: "Google Config",
    args: ["google", "config", "--help"],
    language: "text"
  }
];

function runCli(args: string[]): string {
  const result = spawnSync("cargo", ["run", "-q", "-p", "agent_ads_cli", "--", ...args], {
    cwd: root,
    encoding: "utf8"
  });

  if (result.status !== 0) {
    const detail = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(`failed to run agent-ads ${args.join(" ")}\n${detail}`);
  }

  return result.stdout.trimEnd();
}

const lines: string[] = [
  "# Command Topics",
  "",
  "This file is the generated exhaustive CLI reference.",
  "It is not the primary agent entrypoint.",
  "",
  "Agents should start with `skills/agent-ads/SKILL.md`.",
  "Humans should usually start with `README.md` or `agent-ads --help`.",
  "",
  "Regenerate it with `npm run docs:generate`.",
  ""
];

for (const section of sections) {
  lines.push(`## ${section.title}`, "");
  if (section.intro) {
    lines.push(section.intro, "");
  }
  lines.push("```bash");
  lines.push(`agent-ads ${section.args.join(" ")}`.trim());
  lines.push("```", "");
  lines.push(`\`\`\`${section.language ?? "text"}`);
  lines.push(runCli(section.args));
  lines.push("```", "");
}

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, lines.join("\n"));
