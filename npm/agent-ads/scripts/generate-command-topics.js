#!/usr/bin/env node
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const node_fs_1 = __importDefault(require("node:fs"));
const node_path_1 = __importDefault(require("node:path"));
const node_child_process_1 = require("node:child_process");
const root = node_path_1.default.resolve(__dirname, "..", "..", "..");
const outputPath = node_path_1.default.join(root, "docs", "command-topics.md");
const sections = [
    {
        title: "Root Help",
        intro: "Canonical syntax uses space-separated subcommands. Colon-delimited forms are intentionally undocumented and unsupported.",
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
        intro: "Tracking and measurement-health commands stay provider-specific; there is no shared cross-provider analytics abstraction.",
        args: ["meta", "pixel-health", "--help"],
        language: "text"
    },
    {
        title: "Meta Config",
        args: ["meta", "config", "--help"],
        language: "text"
    },
    {
        title: "Google Placeholder",
        intro: "Google is an explicit namespace, but it is not implemented yet.",
        args: ["google"],
        language: "json"
    },
    {
        title: "TikTok Placeholder",
        intro: "TikTok is an explicit namespace, but it is not implemented yet.",
        args: ["tiktok"],
        language: "json"
    }
];
function runCli(args) {
    const result = (0, node_child_process_1.spawnSync)("cargo", ["run", "-q", "-p", "agent_ads_cli", "--", ...args], {
        cwd: root,
        encoding: "utf8"
    });
    if (result.status !== 0) {
        const detail = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
        throw new Error(`failed to run agent-ads ${args.join(" ")}\n${detail}`);
    }
    return result.stdout.trimEnd();
}
const lines = [
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
node_fs_1.default.mkdirSync(node_path_1.default.dirname(outputPath), { recursive: true });
node_fs_1.default.writeFileSync(outputPath, lines.join("\n"));
