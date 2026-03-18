"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.binaryName = exports.platformPackages = void 0;
exports.resolveBinaryPath = resolveBinaryPath;
const node_fs_1 = __importDefault(require("node:fs"));
const node_os_1 = __importDefault(require("node:os"));
const node_path_1 = __importDefault(require("node:path"));
const platformPackages = {
    "darwin:arm64": {
        packageName: "agent-ads-darwin-arm64",
        workspaceDir: node_path_1.default.join(__dirname, "..", "platform", "darwin-arm64")
    },
    "darwin:x64": {
        packageName: "agent-ads-darwin-x64",
        workspaceDir: node_path_1.default.join(__dirname, "..", "platform", "darwin-x64")
    },
    "linux:arm64": {
        packageName: "agent-ads-linux-arm64-gnu",
        workspaceDir: node_path_1.default.join(__dirname, "..", "platform", "linux-arm64-gnu")
    },
    "linux:x64": {
        packageName: "agent-ads-linux-x64-gnu",
        workspaceDir: node_path_1.default.join(__dirname, "..", "platform", "linux-x64-gnu")
    },
    "win32:x64": {
        packageName: "agent-ads-windows-x64-msvc",
        workspaceDir: node_path_1.default.join(__dirname, "..", "platform", "windows-x64-msvc")
    }
};
exports.platformPackages = platformPackages;
function resolveBinaryPath() {
    const key = `${node_os_1.default.platform()}:${node_os_1.default.arch()}`;
    const target = platformPackages[key];
    if (!target) {
        throw new Error(`Unsupported platform ${key}. No prebuilt agent-ads binary is available.`);
    }
    const sourceCheckoutCandidates = [
        node_path_1.default.join(__dirname, "..", "..", "target", "debug", exports.binaryName),
        node_path_1.default.join(__dirname, "..", "..", "target", "release", exports.binaryName)
    ];
    for (const candidate of sourceCheckoutCandidates) {
        if (node_fs_1.default.existsSync(candidate)) {
            return candidate;
        }
    }
    try {
        const platformPackage = require(target.packageName);
        return platformPackage.binaryPath;
    }
    catch {
        try {
            const workspacePackage = require(target.workspaceDir);
            return workspacePackage.binaryPath;
        }
        catch (workspaceError) {
            throw new Error(`Failed to resolve ${target.packageName}. Reinstall the package or verify that the platform package was published.`, { cause: workspaceError });
        }
    }
}
exports.binaryName = node_os_1.default.platform() === "win32" ? "agent-ads.exe" : "agent-ads";
