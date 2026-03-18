#!/usr/bin/env node
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const node_child_process_1 = require("node:child_process");
const index_1 = require("../index");
const binaryPath = (0, index_1.resolveBinaryPath)();
const result = (0, node_child_process_1.spawnSync)(binaryPath, process.argv.slice(2), {
    stdio: "inherit"
});
if (result.error) {
    throw result.error;
}
process.exit(result.status ?? 1);
