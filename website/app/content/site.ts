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

export const version = "0.8.0";

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
      { to: "/engines/linkedin", label: "LinkedIn" },
      { to: "/engines/x", label: "X Ads" },
    ],
  },
} as const;

export const headerLinks = [
  { href: repoLinks.github, label: "GitHub" },
  { href: repoLinks.npm, label: "NPM" },
] as const;

export const engineOrder: EngineId[] = ["meta", "google", "tiktok", "pinterest", "linkedin", "x"];

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
  linkedin: {
    id: "linkedin",
    name: "LinkedIn",
    eyebrow: "Marketing API",
    description:
      "Ad accounts, campaign groups, campaigns, creatives, and adAnalytics reporting for LinkedIn Marketing API.",
    firstCommand: "agent-ads linkedin ad-accounts list",
    tags: ["Reporting", "Creatives", "Campaign groups"],
    quickStartLead:
      "Use LinkedIn for ad account discovery, campaign hierarchy inspection, creative lookup, and adAnalytics reporting.",
    referenceLead:
      "Auth, account discovery, campaign groups, campaigns, creatives, reporting, and config checks for LinkedIn.",
    stats: [
      { label: "Auth", value: "Access token" },
      { label: "Scope", value: "Ad account IDs" },
      { label: "Best for", value: "B2B reporting and creative audits" },
    ],
  },
  x: {
    id: "x",
    name: "X Ads",
    eyebrow: "Ads API",
    description:
      "Inspect X Ads accounts, campaign objects, promoted tweets, and synchronous or async analytics.",
    firstCommand: "agent-ads x doctor",
    tags: ["Analytics", "Promoted tweets", "OAuth 1.0a"],
    quickStartLead:
      "Use X for ads account discovery, campaign hierarchy inspection, promoted-tweet audits, and X-native analytics workflows.",
    referenceLead:
      "Auth, config, campaign management, creatives, audiences, measurement, and analytics for X Ads.",
    stats: [
      { label: "Auth", value: "4-secret OAuth 1.0a bundle" },
      { label: "Scope", value: "Ads account IDs" },
      { label: "Best for", value: "Campaign diagnostics and analytics jobs" },
    ],
  },
};

export const homeCLIExamples = [
  {
    id: "meta-insights",
    engine: "Meta",
    label: "Spend by age and gender",
    command:
      "$ agent-ads meta insights query \\\n  --account act_12345678 \\\n  --fields spend,impressions,cpc,actions \\\n  --breakdowns age,gender \\\n  --date-preset last_7d",
  },
  {
    id: "google-gaql",
    engine: "Google",
    label: "Campaign cost and conversions",
    command:
      '$ agent-ads google gaql search \\\n  --customer-id 123-456-7890 \\\n  --query "SELECT campaign.name,\n    metrics.cost_micros, metrics.conversions\n    FROM campaign\n    WHERE segments.date DURING LAST_30_DAYS"',
  },
  {
    id: "tiktok-insights",
    engine: "TikTok",
    label: "Campaign-level CPA",
    command:
      "$ agent-ads tiktok insights query \\\n  --advertiser-id 7012345678901234 \\\n  --data-level AUCTION_CAMPAIGN \\\n  --dimensions campaign_id \\\n  --metrics spend,conversion,cost_per_conversion",
  },
  {
    id: "pinterest-targeting",
    engine: "Pinterest",
    label: "Spend by gender and age",
    command:
      "$ agent-ads pinterest targeting-analytics query \\\n  --ad-account-id 549764106178 \\\n  --targeting-type GENDER,AGE_BUCKET \\\n  --columns SPEND,CLICKTHROUGH_1",
  },
  {
    id: "linkedin-analytics",
    engine: "LinkedIn",
    label: "Daily campaign clicks and spend",
    command:
      "$ agent-ads linkedin analytics query \\\n  --finder statistics \\\n  --account-id 1234567890 \\\n  --pivot CAMPAIGN \\\n  --time-granularity DAILY \\\n  --since 2026-03-01 \\\n  --until 2026-03-16 \\\n  --fields impressions,clicks,costInLocalCurrency",
  },
  {
    id: "x-analytics",
    engine: "X",
    label: "Campaign engagement and billing",
    command:
      "$ agent-ads x analytics query \\\n  --account-id 18ce54d4x5t \\\n  --entity campaign \\\n  --entity-id c1234567890 \\\n  --start-time 2026-03-01T00:00:00Z \\\n  --end-time 2026-03-07T00:00:00Z \\\n  --granularity day \\\n  --placement all-on-twitter \\\n  --metric-group engagement,billing",
  },
] as const;

export const homePromptCards = [
  {
    id: "meta-insights",
    category: "Meta Insights",
    prompt: "Break down last week's Meta spend by age and gender",
    command:
      "$ agent-ads meta insights query \\\n  --account act_12345678 \\\n  --fields spend,cpc,actions \\\n  --breakdowns age,gender \\\n  --date-preset last_7d",
  },
  {
    id: "google-gaql",
    category: "Google GAQL",
    prompt: "Which Google campaigns spent the most last month?",
    command:
      '$ agent-ads google gaql search \\\n  --customer-id 123-456-7890 \\\n  --query "SELECT campaign.name,\n    metrics.cost_micros\n    FROM campaign\n    WHERE segments.date DURING LAST_30_DAYS\n    ORDER BY metrics.cost_micros DESC"',
  },
  {
    id: "tiktok-performance",
    category: "TikTok Performance",
    prompt: "Show me cost per conversion for each TikTok campaign",
    command:
      "$ agent-ads tiktok insights query \\\n  --advertiser-id 7012345678901234 \\\n  --data-level AUCTION_CAMPAIGN \\\n  --metrics spend,conversion,cost_per_conversion",
  },
  {
    id: "pinterest-conversions",
    category: "Pinterest Conversions",
    prompt: "How are my Pinterest conversions trending this week?",
    command:
      "$ agent-ads pinterest analytics query \\\n  --ad-account-id 549764106178 \\\n  --columns TOTAL_CONVERSIONS,TOTAL_PAGE_VISIT \\\n  --start-date 2026-03-15 --end-date 2026-03-22",
  },
  {
    id: "linkedin-reporting",
    category: "LinkedIn Reporting",
    prompt: "Show daily LinkedIn campaign clicks and spend for last week",
    command:
      "$ agent-ads linkedin analytics query \\\n  --finder statistics \\\n  --account-id 1234567890 \\\n  --pivot CAMPAIGN \\\n  --time-granularity DAILY \\\n  --since 2026-03-15 \\\n  --until 2026-03-22 \\\n  --fields impressions,clicks,costInLocalCurrency",
  },
  {
    id: "x-reporting",
    category: "X Analytics",
    prompt: "Show last week's X campaign engagement and billing metrics",
    command:
      "$ agent-ads x analytics query \\\n  --account-id 18ce54d4x5t \\\n  --entity campaign \\\n  --entity-id c1234567890 \\\n  --start-time 2026-03-15T00:00:00Z \\\n  --end-time 2026-03-22T00:00:00Z \\\n  --granularity day \\\n  --placement all-on-twitter \\\n  --metric-group engagement,billing",
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
    { num: "01", text: "agent-ads meta insights query \\", highlight: ["agent-ads"] },
    { num: "02", text: '  --account-id "act_12345678" \\', highlight: ["act_12345678"] },
    { num: "03", text: '  --fields "campaign_name,spend,impressions" \\', highlight: ["campaign_name,spend,impressions"] },
    { num: "04", text: "  --date-preset last_7d \\", highlight: ["last_7d"] },
    { num: "05", text: "  --envelope", highlight: ["--envelope"] },
  ],
  comment: "// 1.2k rows in 142ms",
} as const;

// Keep backward-compatible aliases for generated content that uses "providers"
export const providers = engines;
export type ProviderId = EngineId;
export const providerOrder = engineOrder;
