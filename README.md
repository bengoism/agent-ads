# agent-ads

CLI for querying ad platform APIs.

Reports, creatives, accounts, tracking — from the terminal. Built in Rust, shipped via npm. Meta (Facebook/Instagram), Google Ads, TikTok, Pinterest, and LinkedIn supported today, read-only.

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

## Auth Helper

For local setup, start with:

```bash
agent-ads auth
```

This shows which implemented providers are configured and lets you pick one to set up interactively.

For a machine-readable summary:

```bash
agent-ads auth status
```

To clear stored credentials for one provider via the same picker:

```bash
agent-ads auth clear
```

Provider-native auth commands remain canonical for CI and explicit scripting: `agent-ads <provider> auth ...`.

Auth storage now uses one serialized credential-store bundle. If you stored credentials with an older build that wrote one keychain entry per secret, re-run the relevant `auth set` flow once because legacy entries are no longer read.

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

TikTok requires an app ID, app secret, access token, and refresh token. Store the full set once in your OS credential store:

```bash
agent-ads tiktok auth set --full
```

This prompts for app ID, app secret, and access token. Refresh token is optional.

Or set shell variables:

```bash
export TIKTOK_ADS_ACCESS_TOKEN=abc123...
export TIKTOK_ADS_REFRESH_TOKEN=refresh123...
export TIKTOK_ADS_APP_ID=your_app_id
export TIKTOK_ADS_APP_SECRET=your_app_secret
```

TikTok access tokens expire every 24 hours. Refresh with stored or shell-provided app credentials:

```bash
agent-ads tiktok auth refresh
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

## Quick Start (Pinterest)

### 1. Authenticate

Pinterest requires an app ID, app secret, access token, and refresh token. Store them once in your OS credential store:

```bash
agent-ads pinterest auth set
```

Or set shell variables for the current process:

```bash
export PINTEREST_ADS_APP_ID=your_app_id
export PINTEREST_ADS_APP_SECRET=your_app_secret
export PINTEREST_ADS_ACCESS_TOKEN=access-token
export PINTEREST_ADS_REFRESH_TOKEN=refresh-token
```

Refresh with:

```bash
agent-ads pinterest auth refresh
```

### 2. Verify your setup

```bash
agent-ads pinterest doctor
```

Add `--api` to also exchange the refresh token and ping the Pinterest Ads API.

### 3. Discover your ad accounts

```bash
agent-ads pinterest ad-accounts list
```

### 4. Explore campaigns

```bash
agent-ads pinterest campaigns list --ad-account-id 1234567890
```

### 5. Pull a synchronous analytics report

```bash
agent-ads pinterest analytics query \
  --ad-account-id 1234567890 \
  --level campaign \
  --start-date 2026-03-01 \
  --end-date 2026-03-16 \
  --columns IMPRESSION_1,CLICKTHROUGH_1,SPEND_IN_DOLLAR \
  --granularity DAY \
  --campaign-id 987654321
```

### 6. Submit and wait for an async report

```bash
agent-ads pinterest report-runs submit \
  --ad-account-id 1234567890 \
  --level CAMPAIGN \
  --start-date 2026-03-01 \
  --end-date 2026-03-16 \
  --granularity DAY \
  --columns IMPRESSION_1,CLICKTHROUGH_1,SPEND_IN_DOLLAR

agent-ads pinterest report-runs wait \
  --ad-account-id 1234567890 \
  --token report-token
```

## Quick Start (LinkedIn)

### 1. Authenticate

LinkedIn v1 uses an access token only. Store it once in your OS credential store:

```bash
agent-ads linkedin auth set
```

Or set a shell variable for the current process:

```bash
export LINKEDIN_ADS_ACCESS_TOKEN=access-token
```

Optional default account:

```bash
export LINKEDIN_ADS_DEFAULT_ACCOUNT_ID=1234567890
```

### 2. Verify your setup

```bash
agent-ads linkedin doctor
```

Add `--api` to also ping the LinkedIn Marketing API and confirm the token works.

### 3. Discover your ad accounts

```bash
agent-ads linkedin ad-accounts list
agent-ads linkedin ad-accounts search --status ACTIVE
```

### 4. Explore campaigns and creatives

```bash
agent-ads linkedin campaign-groups list --account-id 1234567890
agent-ads linkedin campaigns list --account-id 1234567890
agent-ads linkedin creatives list --account-id 1234567890
```

### 5. Pull a reporting query

```bash
agent-ads linkedin analytics query \
  --finder statistics \
  --account-id 1234567890 \
  --pivot CAMPAIGN \
  --time-granularity DAILY \
  --since 2026-03-01 \
  --until 2026-03-16 \
  --fields impressions,clicks,costInLocalCurrency
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
| `agent-ads auth` | Show aggregated auth status and route into setup |
| `agent-ads auth status` | Show aggregated auth status across implemented providers |
| `agent-ads auth clear` | Pick one provider and clear its stored credentials |
| `agent-ads providers list` | Show available and planned providers |
| `agent-ads meta ...` | Meta Marketing API commands |
| `agent-ads google ...` | Google Ads commands |
| `agent-ads tiktok ...` | TikTok Business API commands |
| `agent-ads pinterest ...` | Pinterest Ads API commands |
| `agent-ads linkedin ...` | LinkedIn Marketing API commands |

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
| `tiktok auth set` | Store the TikTok access token in the OS credential store |
| `tiktok auth set --refresh-token` | Store the TikTok access token and refresh token |
| `tiktok auth set --full` | Store TikTok app credentials, access token, and optional refresh token |
| `tiktok auth status` | Show auth source and secure storage status for all TikTok credentials |
| `tiktok auth delete` | Delete stored TikTok credentials |
| `tiktok auth refresh` | Refresh the TikTok access token using flags, shell env, or stored credentials |
| `tiktok config path` | Show resolved config file path |
| `tiktok config show` | Show full resolved configuration |
| `tiktok config validate` | Validate config file |
| `tiktok doctor` | Verify auth, config, and (optionally) API connectivity |

### Pinterest: Discovery & Objects

| Command | Description |
|---------|-------------|
| `pinterest ad-accounts list` | List ad accounts accessible to your credentials |
| `pinterest ad-accounts get` | Get ad account details |
| `pinterest campaigns list` | List campaigns for an ad account |
| `pinterest adgroups list` | List ad groups for an ad account |
| `pinterest ads list` | List ads for an ad account |
| `pinterest audiences list` | List audiences for an ad account |
| `pinterest audiences get` | Get a single audience |

### Pinterest: Reporting & Analytics

| Command | Description |
|---------|-------------|
| `pinterest analytics query` | Run synchronous analytics queries |
| `pinterest targeting-analytics query` | Break down performance by targeting dimension |
| `pinterest report-runs submit` | Submit an async report request |
| `pinterest report-runs status` | Check async report status |
| `pinterest report-runs wait` | Poll until async report completes |

### Pinterest: Config & Diagnostics

| Command | Description |
|---------|-------------|
| `pinterest auth set` | Store Pinterest credentials in the OS credential store |
| `pinterest auth status` | Show auth source and secure storage status |
| `pinterest auth delete` | Delete stored Pinterest credentials |
| `pinterest auth refresh` | Refresh the Pinterest access token |
| `pinterest config path` | Show resolved config file path |
| `pinterest config show` | Show full resolved configuration |
| `pinterest config validate` | Validate config file |
| `pinterest doctor` | Verify auth, config, and (optionally) API connectivity |

### LinkedIn: Discovery & Objects

| Command | Description |
|---------|-------------|
| `linkedin ad-accounts list` | List accessible ad accounts and join the authenticated user's role |
| `linkedin ad-accounts get` | Get ad account details |
| `linkedin ad-accounts search` | Search ad accounts with LinkedIn-native filters |
| `linkedin campaign-groups list` | List campaign groups for an ad account |
| `linkedin campaigns list` | List campaigns for an ad account |
| `linkedin campaigns get` | Get a single campaign |
| `linkedin creatives list` | List creatives for an ad account |
| `linkedin creatives get` | Get a single creative |

### LinkedIn: Reporting & Diagnostics

| Command | Description |
|---------|-------------|
| `linkedin analytics query` | Run a LinkedIn `adAnalytics` finder query |
| `linkedin auth set` | Store the LinkedIn access token in the OS credential store |
| `linkedin auth status` | Show auth source and secure storage status |
| `linkedin auth delete` | Delete the stored LinkedIn access token |
| `linkedin config path` | Show resolved config file path |
| `linkedin config show` | Show full resolved configuration |
| `linkedin config validate` | Validate config file |
| `linkedin doctor` | Verify auth, config, and (optionally) API connectivity |

## Configuration

### Secrets

Secrets resolve per provider: shell environment first, then OS credential store. `.env` files are not read. Secrets are never read from config files or CLI flags.

Store credentials persistently with `agent-ads auth` for guided local setup, clear one provider with `agent-ads auth clear`, or use `agent-ads <provider> auth set` / `agent-ads <provider> auth delete` for explicit provider flows. Override in CI or one-off sessions with shell env vars (shown in each Quick Start above).

### Non-secret precedence

CLI flags > shell environment > `agent-ads.config.json`

### Config file

Create a config file to set defaults and avoid repeating flags:

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
    },
    "pinterest": {
      "api_base_url": "https://api.pinterest.com",
      "api_version": "v5",
      "timeout_seconds": 60,
      "default_ad_account_id": "1234567890"
    },
    "linkedin": {
      "api_base_url": "https://api.linkedin.com/rest",
      "api_version": "202603",
      "timeout_seconds": 60,
      "default_account_id": "1234567890"
    }
  }
}
```

Setting `default_account_id` (Meta or LinkedIn), `default_customer_id` (Google), `default_advertiser_id` (TikTok), or `default_ad_account_id` (Pinterest) lets you omit those flags from commands.

## Output

Default output is **data-only JSON** on stdout. Errors are JSON on stderr.

- `--format json|jsonl|csv` — choose output format (default: `json`)
- `--output <path>` — write to a file instead of stdout
- `--pretty` — pretty-print JSON
- `--envelope` — wrap data with `{ "data": ..., "meta": ..., "paging": ... }`

Pagination differs by provider (`--all` auto-follows all pages for every provider). Run `agent-ads <provider> --help` for provider-specific flags.

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Transport or internal error |
| 2 | Config or argument error |
| 3 | Meta API error |
| 4 | TikTok API error |
| 5 | Google API error |
| 6 | Pinterest API error |
| 7 | LinkedIn API error |

## Docs Map

| Audience | Start here |
|----------|------------|
| Humans | This README, then `agent-ads --help` |
| AI agents | [skills/agent-ads/SKILL.md](skills/agent-ads/SKILL.md) |
| Meta provider deep-dive | [skills/agent-ads/references/meta.md](skills/agent-ads/references/meta.md) |
| Google provider deep-dive | [skills/agent-ads/references/google.md](skills/agent-ads/references/google.md) |
| TikTok provider deep-dive | [skills/agent-ads/references/tiktok.md](skills/agent-ads/references/tiktok.md) |
| Pinterest provider deep-dive | [skills/agent-ads/references/pinterest.md](skills/agent-ads/references/pinterest.md) |
| LinkedIn provider deep-dive | [skills/agent-ads/references/linkedin.md](skills/agent-ads/references/linkedin.md) |
| Full CLI reference (generated) | [docs/command-topics.md](docs/command-topics.md) |
| Live help | `agent-ads --help`, `agent-ads meta --help`, `agent-ads google --help`, `agent-ads tiktok --help`, `agent-ads pinterest --help`, `agent-ads linkedin --help` |

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
