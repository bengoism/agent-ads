import fs from "node:fs";
import os from "node:os";
import path from "node:path";

type PlatformKey = `${NodeJS.Platform}:${NodeJS.Architecture}`;

type PlatformTarget = {
  packageName: string;
  workspaceDir: string;
};

const platformPackages: Partial<Record<PlatformKey, PlatformTarget>> = {
  "darwin:arm64": {
    packageName: "agent-ads-darwin-arm64",
    workspaceDir: path.join(__dirname, "..", "platform", "darwin-arm64")
  },
  "darwin:x64": {
    packageName: "agent-ads-darwin-x64",
    workspaceDir: path.join(__dirname, "..", "platform", "darwin-x64")
  },
  "linux:arm64": {
    packageName: "agent-ads-linux-arm64-gnu",
    workspaceDir: path.join(__dirname, "..", "platform", "linux-arm64-gnu")
  },
  "linux:x64": {
    packageName: "agent-ads-linux-x64-gnu",
    workspaceDir: path.join(__dirname, "..", "platform", "linux-x64-gnu")
  },
  "win32:x64": {
    packageName: "agent-ads-win32-x64-msvc",
    workspaceDir: path.join(__dirname, "..", "platform", "win32-x64-msvc")
  }
};

export function resolveBinaryPath(): string {
  const key = `${os.platform()}:${os.arch()}` as PlatformKey;
  const target = platformPackages[key];
  if (!target) {
    throw new Error(
      `Unsupported platform ${key}. No prebuilt agent-ads binary is available.`
    );
  }

  const sourceCheckoutCandidates = [
    path.join(__dirname, "..", "..", "target", "debug", binaryName),
    path.join(__dirname, "..", "..", "target", "release", binaryName)
  ];
  for (const candidate of sourceCheckoutCandidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  try {
    const platformPackage = require(target.packageName) as { binaryPath: string };
    return platformPackage.binaryPath;
  } catch {
    try {
      const workspacePackage = require(target.workspaceDir) as { binaryPath: string };
      return workspacePackage.binaryPath;
    } catch (workspaceError) {
      throw new Error(
        `Failed to resolve ${target.packageName}. Reinstall the package or verify that the platform package was published.`,
        { cause: workspaceError }
      );
    }
  }
}

export { platformPackages };

export const binaryName = os.platform() === "win32" ? "agent-ads.exe" : "agent-ads";
