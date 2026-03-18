#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

type PackageTarget = {
  dir: string;
  access: "public";
};

type PackageManifest = {
  name: string;
  version: string;
};

const root = path.resolve(__dirname, "..", "..", "..");
const hasNodeAuthToken = Boolean(process.env.NODE_AUTH_TOKEN);

if (hasNodeAuthToken) {
  console.log("Using NODE_AUTH_TOKEN for npm publish authentication.");
} else {
  console.log(
    "NODE_AUTH_TOKEN is not set. Expecting npm trusted publishing to already be configured for every package."
  );
}

const packages: PackageTarget[] = [
  { dir: path.join(root, "npm", "platform", "darwin-arm64"), access: "public" },
  { dir: path.join(root, "npm", "platform", "darwin-x64"), access: "public" },
  { dir: path.join(root, "npm", "platform", "linux-arm64-gnu"), access: "public" },
  { dir: path.join(root, "npm", "platform", "linux-x64-gnu"), access: "public" },
  { dir: path.join(root, "npm", "platform", "win32-x64-msvc"), access: "public" },
  { dir: path.join(root, "npm", "agent-ads"), access: "public" }
];

function readManifest(packageDir: string): PackageManifest {
  const manifestPath = path.join(packageDir, "package.json");
  return JSON.parse(fs.readFileSync(manifestPath, "utf8")) as PackageManifest;
}

function isPublished(pkg: PackageManifest): boolean {
  const spec = `${pkg.name}@${pkg.version}`;
  try {
    execFileSync("npm", ["view", spec, "version", "--json"], {
      stdio: ["ignore", "pipe", "ignore"],
      encoding: "utf8"
    });
    return true;
  } catch {
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
    execFileSync(
      "npm",
      ["publish", "--access", target.access, "--provenance"],
      {
        cwd: target.dir,
        stdio: "inherit"
      }
    );
  } catch (error) {
    const bootstrapHint = hasNodeAuthToken
      ? "Verify that the token can create packages under the target npm account."
      : "For a first release, add the NPM_PUBLISH_TOKEN GitHub Actions secret or publish the packages once manually before relying on trusted publishing.";
    throw new Error(
      `Failed to publish ${pkg.name}@${pkg.version}. ${bootstrapHint}`,
      { cause: error }
    );
  }
}
