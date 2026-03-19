# agent-ads

Unix-first multi-provider ads CLI for analysts, agents, and CI jobs.

Query ad accounts, pull performance reports, inspect creatives, and diagnose tracking health — all from the terminal. Built in Rust with prebuilt native binaries.

## Install

```bash
npm install -g agent-ads
```

## Quick Start

```bash
# Store your Meta access token once
agent-ads meta auth set

# Or use a shell override for CI / one-off runs
export META_ADS_ACCESS_TOKEN=EAABs...

# Verify your setup
agent-ads meta doctor

# List your businesses and ad accounts
agent-ads meta businesses list
agent-ads meta ad-accounts list --business-id 1234567890

# Pull a performance report
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --date-preset last_7d
```

## What's Supported

- **Meta (Facebook/Instagram)**: businesses, ad accounts, campaigns, ad sets, ads, insights (sync + async), creatives, activities, pixels, datasets, pixel health
- **Google Ads**: customers, hierarchies, campaigns, ad groups, ads, native GAQL search/search-stream, auth/config/doctor
- **TikTok Ads**: advertisers, campaigns, ad groups, ads, reporting, creatives, pixels, audiences, auth/config/doctor

## Key Features

- Provider-explicit commands (`agent-ads meta ...`) — no leaky abstractions
- Data-only JSON on stdout by default (pipe to `jq`, redirect to files)
- CSV and JSONL output formats
- Cursor-based auto-pagination (`--all`, `--max-items`)
- Async report support (`--async --wait`)
- OS credential store for persistent auth, plus shell env override for CI

## Full Documentation

- [GitHub repo](https://github.com/bengoism/agent-ads)
- [Agent skill](https://github.com/bengoism/agent-ads/tree/main/skills/agent-ads)
- [Meta reference docs](https://github.com/bengoism/agent-ads/tree/main/skills/agent-ads/references)
- [Google reference docs](https://github.com/bengoism/agent-ads/tree/main/skills/agent-ads/references/google.md)

## License

MIT
