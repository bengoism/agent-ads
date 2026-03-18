#!/usr/bin/env node
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const node_fs_1 = __importDefault(require("node:fs"));
const node_path_1 = __importDefault(require("node:path"));
const root = node_path_1.default.resolve(__dirname, "..", "..", "..");
const version = process.env.AGENT_ADS_VERSION || "0.1.0";
const binaryDir = process.env.AGENT_ADS_BINARY_DIR || node_path_1.default.join(root, "dist");
const targets = [
    {
        packageDir: node_path_1.default.join(root, "npm", "platform", "darwin-arm64"),
        binaryName: "agent-ads",
        source: "agent-ads-darwin-arm64"
    },
    {
        packageDir: node_path_1.default.join(root, "npm", "platform", "darwin-x64"),
        binaryName: "agent-ads",
        source: "agent-ads-darwin-x64"
    },
    {
        packageDir: node_path_1.default.join(root, "npm", "platform", "linux-arm64-gnu"),
        binaryName: "agent-ads",
        source: "agent-ads-linux-arm64-gnu"
    },
    {
        packageDir: node_path_1.default.join(root, "npm", "platform", "linux-x64-gnu"),
        binaryName: "agent-ads",
        source: "agent-ads-linux-x64-gnu"
    },
    {
        packageDir: node_path_1.default.join(root, "npm", "platform", "windows-x64-msvc"),
        binaryName: "agent-ads.exe",
        source: "agent-ads-win32-x64-msvc.exe"
    }
];
for (const target of targets) {
    const binarySource = node_path_1.default.join(binaryDir, target.source);
    if (!node_fs_1.default.existsSync(binarySource)) {
        continue;
    }
    const binDir = node_path_1.default.join(target.packageDir, "bin");
    node_fs_1.default.mkdirSync(binDir, { recursive: true });
    const destination = node_path_1.default.join(binDir, target.binaryName);
    node_fs_1.default.copyFileSync(binarySource, destination);
    node_fs_1.default.chmodSync(destination, 0o755);
    const packageJsonPath = node_path_1.default.join(target.packageDir, "package.json");
    const packageJson = JSON.parse(node_fs_1.default.readFileSync(packageJsonPath, "utf8"));
    packageJson.version = version;
    node_fs_1.default.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + "\n");
}
const rootPackagePath = node_path_1.default.join(root, "npm", "agent-ads", "package.json");
const rootPackage = JSON.parse(node_fs_1.default.readFileSync(rootPackagePath, "utf8"));
rootPackage.version = version;
for (const packageName of Object.keys(rootPackage.optionalDependencies)) {
    rootPackage.optionalDependencies[packageName] = version;
}
node_fs_1.default.writeFileSync(rootPackagePath, JSON.stringify(rootPackage, null, 2) + "\n");
