# agent-ads

CLI for querying ad platform APIs.

Reports, creatives, accounts, tracking — from the terminal. Built in Rust, shipped via npm. Meta (Facebook/Instagram), Google Ads, and TikTok supported today, read-only.

## Install

Global:

```bash
npm install -g agent-ads
```

Pinned in a project:

```bash
npm install agent-ads
```

From source:

```bash
git clone https://github.com/bengoism/agent-ads
cd agent-ads
npm install && npm run build:ts
cargo build
```

The npm package is a thin CLI launcher. There is no supported JavaScript API.

## Quick Start (Meta)

### 1. Authenticate

Meta requires an access token. Store it once in your OS credential store:

```bash
agent-ads meta auth set
```

For CI, headless Linux, or one-off overrides, you can still set a shell variable for the current process:

```bash
export META_ADS_ACCESS_TOKEN=EAABs...
```

**Required permission:** `ads_read` — read access to campaigns, insights, creatives, and pixels.

**Optional permission:** `business_management` — discover businesses and ad accounts (`businesses list`, `ad-accounts list`). Without it, you can still query any account directly if you know its ID.

Both permissions are read-only. The CLI never creates, modifies, or deletes anything.

Generate a token at the [Graph API Explorer](https://developers.facebook.com/tools/explorer/) — select your app, add the permissions above, and click "Generate Access Token".

On Linux, persistent secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet. If secure storage is unavailable, use `META_ADS_ACCESS_TOKEN` in the shell for that session.

### 2. Verify your setup

```bash
agent-ads meta doctor
```

```json
{
  "ok": true,
  "checks": [
    { "name": "credential_store", "ok": true, "detail": "stored Meta token found in OS credential store" },
    { "name": "config_file", "ok": true, "detail": "using /work/agent-ads.config.json" },
    { "name": "access_token", "ok": true, "detail": "using stored Meta token from the OS credential store" }
  ]
}
```

Add `--api` to also ping the Meta API and confirm your token works.

### 3. Discover your accounts

```bash
# List businesses you have access to
agent-ads meta businesses list

# List ad accounts under a business
agent-ads meta ad-accounts list --business-id 1234567890
```

```json
[
  { "id": "act_1234567890", "name": "My Ad Account", "account_status": 1, "currency": "USD" }
]
```

### 4. Explore campaigns

```bash
agent-ads meta campaigns list --account act_1234567890
```

### 5. Pull a performance report

```bash
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --since 2026-03-01 \
  --until 2026-03-16 \
  --time-increment 1
```

```json
[
  {
    "campaign_id": "120210123456",
    "campaign_name": "Spring Sale",
    "impressions": "4520",
    "clicks": "312",
    "spend": "48.75",
    "date_start": "2026-03-01",
    "date_stop": "2026-03-01"
  }
]
```

### 6. Export to CSV

```bash
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --date-preset last_7d \
  --format csv \
  --output report.csv
```

## Quick Start (TikTok)

### 1. Authenticate

TikTok requires an access token and app credentials. Store the token in your OS credential store:

```bash
agent-ads tiktok auth set
```

Or set shell variables:

```bash
export TIKTOK_ADS_ACCESS_TOKEN=abc123...
export TIKTOK_ADS_APP_ID=your_app_id
export TIKTOK_ADS_APP_SECRET=your_app_secret
```

TikTok tokens expire every 24 hours. Refresh with:

```bash
agent-ads tiktok auth refresh \
  --app-id $TIKTOK_ADS_APP_ID \
  --app-secret $TIKTOK_ADS_APP_SECRET
```

### 2. Verify your setup

```bash
agent-ads tiktok doctor
```

Add `--api` to also ping the TikTok API and confirm your token works.

### 3. Discover your advertisers

```bash
agent-ads tiktok advertisers list \
  --app-id $TIKTOK_ADS_APP_ID \
  --app-secret $TIKTOK_ADS_APP_SECRET
```

### 4. Explore campaigns

```bash
agent-ads tiktok campaigns list --advertiser-id 1234567890
```

### 5. Pull a performance report

```bash
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions campaign_id \
  --metrics spend,impressions,clicks,ctr \
  --start-date 2026-03-01 \
  --end-date 2026-03-16
```

### 6. Export to CSV

```bash
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions campaign_id \
  --metrics spend,impressions,clicks \
  --start-date 2026-03-01 \
  --end-date 2026-03-16 \
  --format csv \
  --output report.csv
```

## Quick Start (Google)

### 1. Authenticate

Google Ads requires a developer token, OAuth client ID, OAuth client secret, and OAuth refresh token. Store them once in your OS credential store:

```bash
agent-ads google auth set
```

Or set shell variables for the current process:

```bash
export GOOGLE_ADS_DEVELOPER_TOKEN=devtoken...
export GOOGLE_ADS_CLIENT_ID=client-id.apps.googleusercontent.com
export GOOGLE_ADS_CLIENT_SECRET=client-secret
export GOOGLE_ADS_REFRESH_TOKEN=refresh-token
```

Optional defaults:

```bash
export GOOGLE_ADS_DEFAULT_CUSTOMER_ID=1234567890
export GOOGLE_ADS_LOGIN_CUSTOMER_ID=1112223333
```

### 2. Verify your setup

```bash
agent-ads google doctor
```

Add `--api` to exchange the refresh token and ping the Google Ads API.

### 3. Discover your customers

```bash
agent-ads google customers list
agent-ads google customers hierarchy --customer-id 1234567890
```

### 4. Explore campaigns

```bash
agent-ads google campaigns list --customer-id 1234567890
```

### 5. Run a GAQL query

```bash
agent-ads google gaql search \
  --customer-id 1234567890 \
  --query "SELECT campaign.id, campaign.name, metrics.impressions, metrics.clicks FROM campaign"
```

### 6. Stream to CSV

```bash
agent-ads google gaql search-stream \
  --customer-id 1234567890 \
  --query-file campaign-query.sql \
  --format csv \
  --output google-report.csv
```

## Command Overview

Providers are always explicit: `agent-ads <provider> <command>`. There is no cross-provider abstraction.

### Top-level

| Command | Description |
|---------|-------------|
| `agent-ads providers list` | Show available and planned providers |
| `agent-ads meta ...` | Meta Marketing API commands |
| `agent-ads google ...` | Google Ads commands |
| `agent-ads tiktok ...` | TikTok Business API commands |

### Meta: Discovery

| Command | Description |
|---------|-------------|
| `meta businesses list` | List businesses accessible to your token |
| `meta ad-accounts list` | List ad accounts under a business |
| `meta campaigns list` | List campaigns in an ad account |
| `meta adsets list` | List ad sets in an ad account |
| `meta ads list` | List ads in an ad account |

### Meta: Reporting

| Command | Description |
|---------|-------------|
| `meta insights query` | Synchronous insights query |
| `meta insights export` | Insights with optional async mode (`--async --wait`) |
| `meta report-runs submit` | Submit an async report job |
| `meta report-runs status` | Check async report status |
| `meta report-runs wait` | Poll until async report completes |
| `meta report-runs results` | Fetch completed report results |

### Meta: Creative & Changes

| Command | Description |
|---------|-------------|
| `meta creatives get` | Fetch a creative by ID |
| `meta creatives preview` | Get rendered ad preview (by creative or ad ID) |
| `meta activities list` | List account activity/change history |

### Meta: Tracking & Measurement

| Command | Description |
|---------|-------------|
| `meta custom-conversions list` | List custom conversions |
| `meta pixels list` | List pixels |
| `meta datasets get` | Get dataset quality metrics |
| `meta pixel-health get` | Combined pixel diagnostics (metadata + stats) |

### Meta: Config & Diagnostics

| Command | Description |
|---------|-------------|
| `meta auth set` | Store the Meta token in the OS credential store |
| `meta auth status` | Show auth source and secure storage status |
| `meta auth delete` | Delete the stored Meta token |
| `meta config path` | Show resolved config file path |
| `meta config show` | Show full resolved configuration |
| `meta config validate` | Validate config file |
| `meta doctor` | Verify auth, config, and (optionally) API connectivity |

### Google: Discovery & Objects

| Command | Description |
|---------|-------------|
| `google customers list` | List customers accessible to your credentials |
| `google customers hierarchy` | Explore a customer hierarchy via `customer_client` |
| `google campaigns list` | List campaigns for a customer |
| `google adgroups list` | List ad groups for a customer |
| `google ads list` | List ads for a customer |

### Google: GAQL & Diagnostics

| Command | Description |
|---------|-------------|
| `google gaql search` | Run a paged GAQL search |
| `google gaql search-stream` | Run a streamed GAQL search |
| `google auth set` | Store Google Ads credentials in the OS credential store |
| `google auth status` | Show auth source and secure storage status |
| `google auth delete` | Delete stored Google Ads credentials |
| `google config path` | Show resolved config file path |
| `google config show` | Show full resolved configuration |
| `google config validate` | Validate config file |
| `google doctor` | Verify auth, config, and (optionally) API connectivity |

### TikTok: Discovery

| Command | Description |
|---------|-------------|
| `tiktok advertisers list` | List advertisers accessible to your token |
| `tiktok advertisers info` | Get advertiser details |
| `tiktok campaigns list` | List campaigns for an advertiser |
| `tiktok adgroups list` | List ad groups for an advertiser |
| `tiktok ads list` | List ads for an advertiser |

### TikTok: Reporting

| Command | Description |
|---------|-------------|
| `tiktok insights query` | Synchronous reporting query |
| `tiktok report-runs submit` | Submit an async report task |
| `tiktok report-runs status` | Check async report task status |
| `tiktok report-runs cancel` | Cancel an async report task |

### TikTok: Creative & Tracking

| Command | Description |
|---------|-------------|
| `tiktok creatives videos` | List video creative assets |
| `tiktok creatives images` | List image creative assets |
| `tiktok pixels list` | List pixels for an advertiser |
| `tiktok audiences list` | List custom audiences |

### TikTok: Config & Diagnostics

| Command | Description |
|---------|-------------|
| `tiktok auth set` | Store the TikTok token in the OS credential store |
| `tiktok auth status` | Show auth source and secure storage status |
| `tiktok auth delete` | Delete the stored TikTok token |
| `tiktok auth refresh` | Refresh the TikTok access token |
| `tiktok config path` | Show resolved config file path |
| `tiktok config show` | Show full resolved configuration |
| `tiktok config validate` | Validate config file |
| `tiktok doctor` | Verify auth, config, and (optionally) API connectivity |

## Global Flags

These flags work with any command:

| Flag | Description |
|------|-------------|
| `--config <path>` | Config file path (default: `agent-ads.config.json`) |
| `--api-base-url <url>` | Override the active provider API base URL |
| `--api-version <version>` | Override the active provider API version (e.g. Meta `v25.0`, Google `v23`) |
| `--timeout-seconds <n>` | HTTP request timeout |
| `--format json\|jsonl\|csv` | Output format (default: `json`) |
| `--output <path>` | Write output to file instead of stdout (`-` for stdout) |
| `--pretty` | Pretty-print JSON output |
| `--envelope` | Include response metadata, paging info, and warnings |
| `--include-meta` | Add metadata columns to CSV output |
| `-q, --quiet` | Suppress warnings and non-data output |
| `-v, --verbose` | Increase log verbosity (repeat for debug: `-vv`) |

## Pagination

Pagination differs by provider.

### Meta (cursor-based)

| Flag | Description |
|------|-------------|
| `--page-size <n>` | Number of items per API request |
| `--cursor <token>` | Resume from a specific cursor |
| `--all` | Automatically follow all pages |
| `--max-items <n>` | Stop after collecting N total items |

Use `--all` to fetch everything, or `--max-items 100` to cap results. Without either flag, you get one page of results and can use `--envelope` to see the paging cursor for the next page.

### Google (page-token)

| Flag | Description |
|------|-------------|
| `--page-size <n>` | Number of rows per API page |
| `--page-token <token>` | Resume from a Google `nextPageToken` |
| `--all` | Automatically follow all pages |
| `--max-items <n>` | Stop after collecting N total rows |

### TikTok (page-number)

| Flag | Description |
|------|-------------|
| `--page-size <n>` | Number of items per page |
| `--page <n>` | Page number (1-indexed) |
| `--all` | Automatically follow all pages |
| `--max-items <n>` | Stop after collecting N total items |

## Configuration

### Secret resolution

Secrets resolve in this order (per provider):

1. Shell environment (for example `META_ADS_ACCESS_TOKEN` or `GOOGLE_ADS_REFRESH_TOKEN`)
2. OS credential store (`agent-ads <provider> auth set`)

`.env` files are not read.

### Non-secret precedence

CLI flags > shell environment > `agent-ads.config.json`

### Config file

Copy the example to get started:

```bash
cp agent-ads.config.json.example agent-ads.config.json
```

Supported keys:

```json
{
  "output_format": "json",
  "providers": {
    "meta": {
      "api_base_url": "https://graph.facebook.com",
      "api_version": "v25.0",
      "timeout_seconds": 60,
      "default_business_id": "1234567890",
      "default_account_id": "act_1234567890"
    },
    "google": {
      "api_base_url": "https://googleads.googleapis.com",
      "api_version": "v23",
      "timeout_seconds": 60,
      "default_customer_id": "1234567890",
      "login_customer_id": "1112223333"
    },
    "tiktok": {
      "api_base_url": "https://business-api.tiktok.com",
      "api_version": "v1.3",
      "timeout_seconds": 60,
      "default_advertiser_id": "1234567890"
    }
  }
}
```

Setting `default_business_id` / `default_account_id` (Meta), `default_customer_id` / `login_customer_id` (Google), or `default_advertiser_id` (TikTok) lets you omit those flags from commands.

### Environment variables

| Variable | Provider | Required | Description |
|----------|----------|----------|-------------|
| `META_ADS_ACCESS_TOKEN` | Meta | No | Shell override / CI fallback for the Meta API access token |
| `GOOGLE_ADS_DEVELOPER_TOKEN` | Google | No | Shell override / CI fallback for the Google Ads developer token |
| `GOOGLE_ADS_CLIENT_ID` | Google | No | Shell override / CI fallback for the Google OAuth client ID |
| `GOOGLE_ADS_CLIENT_SECRET` | Google | No | Shell override / CI fallback for the Google OAuth client secret |
| `GOOGLE_ADS_REFRESH_TOKEN` | Google | No | Shell override / CI fallback for the Google OAuth refresh token |
| `GOOGLE_ADS_DEFAULT_CUSTOMER_ID` | Google | No | Default customer ID for Google customer-scoped commands |
| `GOOGLE_ADS_LOGIN_CUSTOMER_ID` | Google | No | Manager account header for Google customer-scoped commands |
| `TIKTOK_ADS_ACCESS_TOKEN` | TikTok | No | Shell override / CI fallback for the TikTok API access token |
| `TIKTOK_ADS_REFRESH_TOKEN` | TikTok | No | Refresh token for `auth refresh` flow |
| `TIKTOK_ADS_APP_ID` | TikTok | For `advertisers list` and `auth refresh` | TikTok app ID |
| `TIKTOK_ADS_APP_SECRET` | TikTok | For `advertisers list` and `auth refresh` | TikTok app secret |

Secrets are never read from config files. Runtime auth resolution uses shell environment variables or the OS credential store; `agent-ads google auth set` can also accept CLI flags when seeding the credential store.

## Output

Default output is **data-only JSON** on stdout. Errors are JSON on stderr.

### Formats

```bash
# Default: JSON array
agent-ads meta businesses list
# [{"id":"123","name":"My Business"}]

# One JSON object per line (good for streaming/piping)
agent-ads meta businesses list --format jsonl
# {"id":"123","name":"My Business"}

# CSV (good for spreadsheets)
agent-ads meta businesses list --format csv
# id,name
# 123,My Business

# Pretty-printed JSON
agent-ads meta businesses list --pretty
```

### Envelope mode

Add `--envelope` to wrap data with response metadata, provider-native paging info, and warnings:

```bash
agent-ads meta businesses list --envelope --pretty
```

```json
{
  "data": [{ "id": "123", "name": "My Business" }],
  "meta": { "api_version": "v25.0", "endpoint": "/me/businesses" },
  "paging": { "cursors": { "before": "...", "after": "..." }, "next": "..." }
}
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Transport or internal error |
| 2 | Config or argument error |
| 3 | Meta API error |
| 4 | TikTok API error |
| 5 | Google API error |

## Docs Map

| Audience | Start here |
|----------|------------|
| Humans | This README, then `agent-ads --help` |
| AI agents | [skills/agent-ads/SKILL.md](skills/agent-ads/SKILL.md) |
| Meta provider deep-dive | [skills/agent-ads/references/meta.md](skills/agent-ads/references/meta.md) |
| Google provider deep-dive | [skills/agent-ads/references/google.md](skills/agent-ads/references/google.md) |
| TikTok provider deep-dive | [skills/agent-ads/references/tiktok.md](skills/agent-ads/references/tiktok.md) |
| Full CLI reference (generated) | [docs/command-topics.md](docs/command-topics.md) |
| Live help | `agent-ads --help`, `agent-ads meta --help`, `agent-ads google --help`, `agent-ads tiktok --help` |

## Skills

If you use this repo with an agent runtime (Claude Code, Codex, etc.), install the public skill:

```bash
npx skills add bengoism/agent-ads
```

The repo-local source of truth for the skill is [skills/agent-ads/SKILL.md](skills/agent-ads/SKILL.md).

## Development

```bash
npm install
npm run build:ts
cargo fmt
cargo test
npm run test:smoke
npm run docs:generate

# Run locally from source
cargo run -p agent_ads_cli -- --help
cargo run -p agent_ads_cli -- meta doctor
```

## Publishing

The release workflow publishes six npm packages:

- `agent-ads` — CLI launcher
- `agent-ads-darwin-arm64` / `agent-ads-darwin-x64` — macOS binaries
- `agent-ads-linux-arm64-gnu` / `agent-ads-linux-x64-gnu` — Linux binaries
- `agent-ads-windows-x64-msvc` — Windows binary

For a brand-new release, bootstrap with an npm publish token first. After the first successful publish, switch the packages to [npm trusted publishing](https://docs.npmjs.com/generating-provenance-statements):

```bash
npm run release:trust
```

## License

MIT
