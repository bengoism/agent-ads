import { generatedContent } from "../generated/content";

export type EngineId = keyof typeof generatedContent.quickStarts;

export type EngineMeta = {
  id: EngineId;
  name: string;
  eyebrow: string;
  description: string;
  firstCommand: string;
  tags: string[];
  quickStartLead: string;
  referenceLead: string;
  stats: Array<{ label: string; value: string }>;
};

export const version = "0.7.0";

export const repoLinks = {
  github: "https://github.com/bengoism/agent-ads",
  npm: "https://www.npmjs.com/package/agent-ads",
};

export const syntaxRule = "agent-ads <provider> <command>";

export const sidebarNav = {
  navigation: {
    label: "NAVIGATION",
    subtitle: `Documentation v${version}`,
    items: [
      { to: "/", label: "Quick Start", end: true },
      { to: "/auth", label: "Auth" },
      { to: "/config", label: "Config" },
    ],
  },
  engines: {
    label: "ENGINES",
    items: [
      { to: "/engines/meta", label: "Meta Ads" },
      { to: "/engines/google", label: "Google Ads" },
      { to: "/engines/tiktok", label: "TikTok" },
      { to: "/engines/pinterest", label: "Pinterest" },
    ],
  },
} as const;

export const headerLinks = [
  { href: repoLinks.github, label: "GitHub" },
  { href: repoLinks.npm, label: "NPM" },
] as const;

export const engineOrder: EngineId[] = ["meta", "google", "tiktok", "pinterest"];

export const engines: Record<EngineId, EngineMeta> = {
  meta: {
    id: "meta",
    name: "Meta Ads",
    eyebrow: "Marketing API",
    description:
      "Full Graph API integration. Fetch creative assets, campaign stats, and breakdown insights.",
    firstCommand: "agent-ads meta doctor",
    tags: ["Campaigns", "Creatives"],
    quickStartLead:
      "Use Meta for business discovery, account-scoped reporting, creative inspection, change history, and tracking diagnostics.",
    referenceLead:
      "Auth, account objects, reports, creatives, tracking diagnostics, and workflow recipes for the Meta command surface.",
    stats: [
      { label: "Auth", value: "Access token plus ads_read" },
      { label: "Scope", value: "Business IDs and act_* accounts" },
      { label: "Best for", value: "Insights, creatives, pixel health" },
    ],
  },
  google: {
    id: "google",
    name: "Google Ads",
    eyebrow: "GAQL-native",
    description:
      "Support for Google Ads Query Language (GAQL). Stream large report batches to local disk.",
    firstCommand: "agent-ads google customers list",
    tags: ["GAQL", "IRC"],
    quickStartLead:
      "Use Google Ads for customer discovery, hierarchy inspection, and GAQL queries or exports with explicit customer scope.",
    referenceLead:
      "Auth, customer discovery, GAQL behavior, and query workflows for Google Ads.",
    stats: [
      { label: "Auth", value: "Developer token plus OAuth refresh token" },
      { label: "Scope", value: "Customer IDs and GAQL" },
      { label: "Best for", value: "GAQL, hierarchy, exports" },
    ],
  },
  tiktok: {
    id: "tiktok",
    name: "TikTok",
    eyebrow: "Business API",
    description:
      "Query business API for performance data. Optimized for high-volume creative asset retrieval.",
    firstCommand:
      "agent-ads tiktok advertisers list --app-id $TIKTOK_ADS_APP_ID --app-secret $TIKTOK_ADS_APP_SECRET",
    tags: ["Performance", "Assets"],
    quickStartLead:
      "Use TikTok for advertiser discovery, performance reporting, creative asset lookup, pixels, and audiences.",
    referenceLead:
      "Auth, advertiser-scoped reports, creative assets, tracking surfaces, and recurring TikTok workflows.",
    stats: [
      { label: "Auth", value: "App credentials plus 24h token" },
      { label: "Scope", value: "Advertiser IDs" },
      { label: "Best for", value: "Advertiser-scoped insights and assets" },
    ],
  },
  pinterest: {
    id: "pinterest",
    name: "Pinterest",
    eyebrow: "Ads API",
    description:
      "Ad accounts, synchronous analytics, targeting analytics, audiences, and async report runs for Pinterest Ads API.",
    firstCommand: "agent-ads pinterest ad-accounts list",
    tags: ["Analytics", "Report runs", "Audiences"],
    quickStartLead:
      "Use Pinterest for account discovery, analytics queries, report-run exports, audiences, and targeting breakdowns.",
    referenceLead:
      "Auth, analytics queries, report runs, audiences, and targeting analytics for Pinterest.",
    stats: [
      { label: "Auth", value: "App credentials plus refresh token" },
      { label: "Scope", value: "Ad account IDs" },
      { label: "Best for", value: "Analytics, report runs, audiences" },
    ],
  },
};

export const homePromptCards = [
  {
    id: "tiktok-performance",
    category: "TikTok Performance",
    prompt: "What's my TikTok campaign performance this month?",
    command:
      "$ agent-ads tiktok insights query \\\n  --time-range this_month",
  },
  {
    id: "pixel-health",
    category: "Pixel Health",
    prompt: 'Check if my Meta pixel is working',
    command:
      '$ agent-ads meta pixel-health get \\\n  --pixel-id "pix_98765"',
  },
  {
    id: "google-gaql",
    category: "Google GAQL",
    prompt: "Run a GAQL search for all active Google campaigns",
    command:
      '$ agent-ads google gaql search \\\n  --query "SELECT campaign.name FROM\n  campaign..."',
  },
  {
    id: "pinterest-reports",
    category: "Pinterest Reports",
    prompt: "Submit a Pinterest report and wait for the results",
    command:
      "$ agent-ads pinterest report-runs submit \\\n  --sync true",
  },
] as const;

export const performanceFeatures = [
  {
    title: "Static Binary",
    description: "Zero runtime dependencies. Deployable in any CI/CD runner.",
  },
  {
    title: "Local SQLite Cache",
    description: "Ultra-fast repeated queries with optional persistent storage.",
  },
] as const;

export const performanceCode = {
  filename: "example.sh",
  lines: [
    { num: "01", text: "agent-ads query meta \\", highlight: ["agent-ads"] },
    { num: "02", text: '  --account-id "act_12345678" \\', highlight: ["act_12345678"] },
    { num: "03", text: '  --fields "name,status,spend" \\', highlight: ["name,status,spend"] },
    { num: "04", text: "  --format json \\", highlight: ["json"] },
    { num: "05", text: "  --output ./reports/daily.json", highlight: ["./reports/daily.json"] },
  ],
  comment: "// Processed 1.2k rows in 142ms",
} as const;

// Keep backward-compatible aliases for generated content that uses "providers"
export const providers = engines;
export type ProviderId = EngineId;
export const providerOrder = engineOrder;
