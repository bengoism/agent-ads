#!/usr/bin/env node
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const node_child_process_1 = require("node:child_process");
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
    console.log(`Configuring npm trusted publishing for ${packageName} from ${repo} using .github/workflows/${workflowFile}`);
    (0, node_child_process_1.execFileSync)("npm", ["trust", "github", packageName, "--repo", repo, "--file", workflowFile, "-y"], {
        stdio: "inherit"
    });
}
