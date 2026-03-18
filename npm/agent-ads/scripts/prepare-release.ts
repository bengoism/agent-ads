#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

type ReleaseTarget = {
  packageDir: string;
  binaryName: string;
  source: string;
};

const root = path.resolve(__dirname, "..", "..", "..");
const version = process.env.AGENT_ADS_VERSION || "0.1.0";
const binaryDir = process.env.AGENT_ADS_BINARY_DIR || path.join(root, "dist");

const targets: ReleaseTarget[] = [
  {
    packageDir: path.join(root, "npm", "platform", "darwin-arm64"),
    binaryName: "agent-ads",
    source: "agent-ads-darwin-arm64"
  },
  {
    packageDir: path.join(root, "npm", "platform", "darwin-x64"),
    binaryName: "agent-ads",
    source: "agent-ads-darwin-x64"
  },
  {
    packageDir: path.join(root, "npm", "platform", "linux-arm64-gnu"),
    binaryName: "agent-ads",
    source: "agent-ads-linux-arm64-gnu"
  },
  {
    packageDir: path.join(root, "npm", "platform", "linux-x64-gnu"),
    binaryName: "agent-ads",
    source: "agent-ads-linux-x64-gnu"
  },
  {
    packageDir: path.join(root, "npm", "platform", "windows-x64-msvc"),
    binaryName: "agent-ads.exe",
    source: "agent-ads-win32-x64-msvc.exe"
  }
];

for (const target of targets) {
  const binarySource = path.join(binaryDir, target.source);
  if (!fs.existsSync(binarySource)) {
    continue;
  }

  const binDir = path.join(target.packageDir, "bin");
  fs.mkdirSync(binDir, { recursive: true });
  const destination = path.join(binDir, target.binaryName);
  fs.copyFileSync(binarySource, destination);
  fs.chmodSync(destination, 0o755);

  const packageJsonPath = path.join(target.packageDir, "package.json");
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")) as {
    version: string;
  };
  packageJson.version = version;
  fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + "\n");
}

const rootPackagePath = path.join(root, "npm", "agent-ads", "package.json");
const rootPackage = JSON.parse(fs.readFileSync(rootPackagePath, "utf8")) as {
  version: string;
  optionalDependencies: Record<string, string>;
};
rootPackage.version = version;
for (const packageName of Object.keys(rootPackage.optionalDependencies)) {
  rootPackage.optionalDependencies[packageName] = version;
}
fs.writeFileSync(rootPackagePath, JSON.stringify(rootPackage, null, 2) + "\n");
