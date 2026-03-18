#!/usr/bin/env node

import { spawnSync } from "node:child_process";

import { resolveBinaryPath } from "../index";

const binaryPath = resolveBinaryPath();
const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit"
});

if (result.error) {
  throw result.error;
}

process.exit(result.status ?? 1);
