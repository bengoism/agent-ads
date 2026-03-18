#!/usr/bin/env node

import { execFileSync, spawnSync } from "node:child_process";

const repo = "bengoism/agent-ads";
const workflowFile = "release.yml";
const trustNpmVersion = "11.11.0";

const packages = [
  "agent-ads-darwin-arm64",
  "agent-ads-darwin-x64",
  "agent-ads-linux-arm64-gnu",
  "agent-ads-linux-x64-gnu",
  "agent-ads-windows-x64-msvc",
  "agent-ads"
];

function resolveNpmCommand(): { command: string; args: string[] } {
  const supportsTrust = spawnSync("npm", ["trust", "--help"], {
    stdio: "ignore"
  }).status === 0;

  if (supportsTrust) {
    return {
      command: "npm",
      args: []
    };
  }

  console.log(
    `Local npm does not support "trust"; falling back to npx npm@${trustNpmVersion}.`
  );

  return {
    command: "npx",
    args: ["--yes", `npm@${trustNpmVersion}`]
  };
}

const npmCommand = resolveNpmCommand();

for (const packageName of packages) {
  console.log(
    `Configuring npm trusted publishing for ${packageName} from ${repo} using .github/workflows/${workflowFile}`
  );
  execFileSync(
    npmCommand.command,
    [...npmCommand.args, "trust", "github", packageName, "--repo", repo, "--file", workflowFile, "-y"],
    {
      stdio: "inherit"
    }
  );
}
