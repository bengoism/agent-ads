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
const hasNodeAuthToken = Boolean(process.env.NODE_AUTH_TOKEN);
if (hasNodeAuthToken) {
    console.log("Using NODE_AUTH_TOKEN for npm publish authentication.");
}
else {
    console.log("NODE_AUTH_TOKEN is not set. Expecting npm trusted publishing to already be configured for every package.");
}
const packages = [
    { dir: node_path_1.default.join(root, "npm", "platform", "darwin-arm64"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "platform", "darwin-x64"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "platform", "linux-arm64-gnu"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "platform", "linux-x64-gnu"), access: "public" },
    { dir: node_path_1.default.join(root, "npm", "platform", "windows-x64-msvc"), access: "public" },
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
    try {
        (0, node_child_process_1.execFileSync)("npm", ["publish", "--access", target.access, "--provenance"], {
            cwd: target.dir,
            stdio: "inherit"
        });
    }
    catch (error) {
        const bootstrapHint = hasNodeAuthToken
            ? "Verify that the token can create packages under the target npm account."
            : "For a first release, add the NPM_PUBLISH_TOKEN GitHub Actions secret or publish the packages once manually before relying on trusted publishing.";
        throw new Error(`Failed to publish ${pkg.name}@${pkg.version}. ${bootstrapHint}`, { cause: error });
    }
}
