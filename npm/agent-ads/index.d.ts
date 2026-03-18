type PlatformKey = `${NodeJS.Platform}:${NodeJS.Architecture}`;
type PlatformTarget = {
    packageName: string;
    workspaceDir: string;
};
declare const platformPackages: Partial<Record<PlatformKey, PlatformTarget>>;
export declare function resolveBinaryPath(): string;
export { platformPackages };
export declare const binaryName: string;
