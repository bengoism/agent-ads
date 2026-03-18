#!/usr/bin/env node

import { execFileSync } from "node:child_process";

const repo = "bengoism/agent-ads";
const workflowFile = "release.yml";

const packages = [
  "agent-ads-darwin-arm64",
  "agent-ads-darwin-x64",
  "agent-ads-linux-arm64-gnu",
  "agent-ads-linux-x64-gnu",
  "agent-ads-win32-x64-msvc",
  "agent-ads"
];

for (const packageName of packages) {
  console.log(
    `Configuring npm trusted publishing for ${packageName} from ${repo} using .github/workflows/${workflowFile}`
  );
  execFileSync(
    "npm",
    ["trust", "github", packageName, "--repo", repo, "--file", workflowFile, "-y"],
    {
      stdio: "inherit"
    }
  );
}
