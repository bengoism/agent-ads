#!/usr/bin/env node
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const node_child_process_1 = require("node:child_process");
const node_fs_1 = __importDefault(require("node:fs"));
const node_path_1 = __importDefault(require("node:path"));
const root = node_path_1.default.resolve(__dirname, "..", "..", "..");
const packages = [
    { dir: node_path_1.default.join(root, "npm", "platform", "darwin-arm64"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "platform", "darwin-x64"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "platform", "linux-arm64-gnu"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "platform", "linux-x64-gnu"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "platform", "win32-x64-msvc"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "agent-ads"), access: "public" }
];
function readManifest(packageDir) {
    const manifestPath = node_path_1.default.join(packageDir, "package.json");
    return JSON.parse(node_fs_1.default.readFileSync(manifestPath, "utf8"));
}
function isPublished(pkg) {
    const spec = `${pkg.name}@${pkg.version}`;
    try {
        (0, node_child_process_1.execFileSync)("npm", ["view", spec, "version", "--json"], {
            stdio: ["ignore", "pipe", "ignore"],
            encoding: "utf8"
        });
        return true;
    }
    catch {
        return false;
    }
}
for (const target of packages) {
    const pkg = readManifest(target.dir);
    if (isPublished(pkg)) {
        console.log(`Skipping ${pkg.name}@${pkg.version}; already published.`);
        continue;
    }
    console.log(`Publishing ${pkg.name}@${pkg.version} from ${target.dir}`);
    (0, node_child_process_1.execFileSync)("npm", ["publish", "--access", target.access, "--provenance"], {
        cwd: target.dir,
        stdio: "inherit"
    });
}
