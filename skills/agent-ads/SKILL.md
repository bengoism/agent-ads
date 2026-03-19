---
name: agent-ads
description: >
  Operate the provider-first ads CLI from the terminal. Use when the user wants
  to inspect available ad providers, route work into the implemented Meta,
  Google, or TikTok providers, or keep ad-provider commands explicit inside
  `agent-ads`.
---

# Agent Ads

`agent-ads` is a Unix-first CLI for querying ad platform APIs. It supports Meta (Facebook/Instagram) Marketing API, Google Ads, and TikTok Business API. All commands are read-only.

This file is the only public skill entrypoint. It routes you to provider-specific reference docs.

## What You Can Ask

Plain-English prompts that work. The agent translates these into the right CLI commands.

**Performance & reporting**
- "Show me my Meta ad spend by campaign for the last 7 days"
- "Run this GAQL query against Google Ads"
- "What's my TikTok campaign performance this month?"
- "Pull a daily breakdown of impressions and clicks across all campaigns"
- "Export last week's Meta performance report to CSV"
- "Run an async report for all my TikTok ads since January"

**Accounts & campaigns**
- "What businesses do I have access to in Meta?"
- "List all my accessible Google Ads customers"
- "List all my TikTok advertisers"
- "Show me the campaigns in my Meta ad account"
- "Which ad sets are running right now?"

**Creative & tracking**
- "Show my TikTok video creatives"
- "Check if my Meta pixel is working"
- "Find broken pixels across my Meta accounts"
- "What audiences do I have in TikTok?"

**Setup & troubleshooting**
- "Help me set up my Meta auth token"
- "Is my TikTok token still valid?"
- "Run a doctor check on my account"
- "My token expired — how do I refresh it?"

## Glossary

Translating marketer language to CLI concepts:

| You say | Platform | In the CLI |
|---------|----------|------------|
| business manager | Meta | `agent-ads meta businesses list` |
| ad account | Meta | `--account act_123...` flag |
| customer / account | Google | `--customer-id 1234567890` flag |
| advertiser / ad account | TikTok | `--advertiser-id 123...` flag |
| campaign performance / report | both | `insights query` command |
| GAQL / query | Google | `gaql search`, `gaql search-stream` |
| pixel / tag | Meta | `pixels list`, `pixel-health get` |
| pixel | TikTok | `pixels list` |
| audience / custom audience | TikTok | `audiences list` |
| creative / ad content | Meta | `creatives get`, `creatives preview` |
| creative / video / image | TikTok | `creatives videos`, `creatives images` |
| conversion tracking | Meta | `custom-conversions list`, `datasets get` |
| token / access token | both | stored via `auth set` or shell env var |

## Command Syntax Rules

Every command starts with `agent-ads <provider>`. This is intentional — each ad platform has different APIs, auth, object models, and semantics, so the CLI keeps them separate rather than papering over differences.

- Canonical: `agent-ads meta insights query ...`
- Never: `agent-ads insights query ...` (no implicit provider)
- Never: `agent-ads meta:insights:query` (no colon syntax)

## Provider Status

| Provider | Status | Summary |
|----------|--------|---------|
| `meta` | Implemented | Read-only Meta Marketing API: accounts, campaigns, insights, creatives, tracking |
| `google` | Implemented | Read-only Google Ads: customers, hierarchies, objects, native GAQL, diagnostics |
| `tiktok` | Implemented | Read-only TikTok Business API: advertisers, campaigns, insights, creatives, pixels, audiences |

Check live: `agent-ads providers list`

## Start Here

| Need | First command | Then read |
|------|---------------|-----------|
| See which providers exist | `agent-ads providers list` | this file |
| Work with Meta (accounts, reports, creatives, tracking) | `agent-ads meta --help` | [references/meta.md](references/meta.md) |
| Work with Google (customers, GAQL, diagnostics) | `agent-ads google --help` | [references/google.md](references/google.md) |
| Work with TikTok (advertisers, campaigns, insights, creatives) | `agent-ads tiktok --help` | [references/tiktok.md](references/tiktok.md) |

## Global Flags

These flags work with every command and every provider:

| Flag | Default | What it does |
|------|---------|-------------|
| `--config <path>` | `agent-ads.config.json` | Config file path |
| `--format json\|jsonl\|csv` | `json` | Output format |
| `--output <path>` | stdout | Write to file (`-` for stdout) |
| `--pretty` | off | Pretty-print JSON |
| `--envelope` | off | Wrap data with metadata, paging, warnings |
| `--include-meta` | off | Add metadata columns in CSV mode |
| `--api-version <v>` | provider default | Provider API version override (Meta `v25.0`, Google `v23`) |
| `--timeout-seconds <n>` | `60` | HTTP request timeout |
| `-q` / `-v` | warn | Quiet mode or verbose logging (`-vv` for debug) |

## Pagination Flags

Pagination differs by provider. Both support `--all` and `--max-items`.

### Meta (cursor-based)

| Flag | What it does |
|------|-------------|
| `--page-size <n>` | Items per API request |
| `--cursor <token>` | Resume from a specific cursor |
| `--all` | Auto-follow all pages |
| `--max-items <n>` | Stop after N items |

### TikTok (page-number)

| Flag | What it does |
|------|-------------|
| `--page-size <n>` | Items per page |
| `--page <n>` | Page number (1-indexed) |
| `--all` | Auto-follow all pages |
| `--max-items <n>` | Stop after N items |

### Google (page-token)

| Flag | What it does |
|------|-------------|
| `--page-size <n>` | Rows per API page |
| `--page-token <token>` | Resume from a Google `nextPageToken` |
| `--all` | Auto-follow all pages |
| `--max-items <n>` | Stop after N rows |

## Output

- **stdout**: data-only JSON by default (just the array or object, no wrapper)
- **stderr**: errors as JSON, warnings as plain text
- **`--envelope`**: wraps stdout with `{ "data": ..., "meta": ..., "paging": ... }`
- **Exit codes**: 0 = success, 1 = transport/internal, 2 = config/argument, 3 = Meta API error, 4 = TikTok API error, 5 = Google API error

## Token Permissions

### Meta

The user's `META_ADS_ACCESS_TOKEN` needs specific Meta permissions (scopes) depending on what commands they run:

| Permission | Needed for |
|------------|------------|
| `ads_read` | All `--account` commands: campaigns, insights, creatives, pixels |
| `business_management` | `businesses list` and `ad-accounts list` (discovery) |

Both are read-only — no write access is granted. If the user gets a "Missing Permission" error, they need to regenerate their token with the correct scopes at the [Graph API Explorer](https://developers.facebook.com/tools/explorer/).

### TikTok

The user's `TIKTOK_ADS_ACCESS_TOKEN` is obtained through TikTok's OAuth flow and expires every 24 hours. Use `agent-ads tiktok auth refresh` to rotate. The `advertisers list` and `auth refresh` commands require app credentials (`TIKTOK_ADS_APP_ID` and `TIKTOK_ADS_APP_SECRET`).

### Google

Google Ads requires four credential pieces:

| Credential | Source |
|------------|--------|
| Developer token | `agent-ads google auth set` or `GOOGLE_ADS_DEVELOPER_TOKEN` |
| OAuth client ID | `agent-ads google auth set` or `GOOGLE_ADS_CLIENT_ID` |
| OAuth client secret | `agent-ads google auth set` or `GOOGLE_ADS_CLIENT_SECRET` |
| OAuth refresh token | `agent-ads google auth set` or `GOOGLE_ADS_REFRESH_TOKEN` |

## Auth Storage

- Persistent auth: `agent-ads meta auth set` / `agent-ads google auth set` / `agent-ads tiktok auth set`
- Shell override / CI fallback: provider-specific env vars such as `META_ADS_ACCESS_TOKEN=...`, `GOOGLE_ADS_REFRESH_TOKEN=...`, or `TIKTOK_ADS_ACCESS_TOKEN=...`
- Linux secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet

## First-Time Setup

If the user has never used `agent-ads` before, walk them through these steps.

### Meta — first run

```bash
# 1. Store your token (get one from https://developers.facebook.com/tools/explorer/)
agent-ads meta auth set

# 2. Verify everything works (--api pings the Meta API)
agent-ads meta doctor --api

# 3. See what you have access to
agent-ads meta businesses list
```

If step 2 fails, check the Token Permissions section above — they likely need `ads_read` and optionally `business_management` scopes.

### TikTok — first run

```bash
# 1. Store your token
agent-ads tiktok auth set

# 2. Verify everything works
agent-ads tiktok doctor --api

# 3. List your advertisers (requires app credentials)
agent-ads tiktok advertisers list \
  --app-id $TIKTOK_ADS_APP_ID \
  --app-secret $TIKTOK_ADS_APP_SECRET
```

TikTok tokens expire every 24 hours. If the doctor check fails with an auth error, refresh with `agent-ads tiktok auth refresh --app-id ... --app-secret ...`.

### Google — first run

```bash
# 1. Store your Google Ads credentials
agent-ads google auth set

# 2. Verify everything works
agent-ads google doctor --api

# 3. List your accessible customers
agent-ads google customers list
```

## Worked Examples

### Meta — discover accounts and pull a report

```bash
# 1. Check setup
agent-ads meta doctor --api

# 2. Find your businesses
agent-ads meta businesses list

# 3. List ad accounts for a business
agent-ads meta ad-accounts list --business-id 1234567890

# 4. List campaigns in an account
agent-ads meta campaigns list --account act_9876543210

# 5. Pull daily performance for the last 7 days
agent-ads meta insights query \
  --account act_9876543210 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --date-preset last_7d \
  --time-increment 1

# 6. Export a large report asynchronously
agent-ads meta insights export \
  --account act_9876543210 \
  --level ad \
  --fields ad_id,ad_name,impressions,clicks,spend \
  --since 2026-01-01 --until 2026-03-01 \
  --async --wait \
  --format csv --output large-report.csv
```

### TikTok — discover advertisers and pull a report

```bash
# 1. Check setup
agent-ads tiktok doctor --api

# 2. List your advertisers
agent-ads tiktok advertisers list \
  --app-id $TIKTOK_ADS_APP_ID \
  --app-secret $TIKTOK_ADS_APP_SECRET

# 3. List campaigns for an advertiser
agent-ads tiktok campaigns list --advertiser-id 1234567890

# 4. Pull campaign performance for a date range
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions campaign_id \
  --metrics spend,impressions,clicks,ctr \
  --start-date 2026-03-01 \
  --end-date 2026-03-16

# 5. Export to CSV
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions campaign_id \
  --metrics spend,impressions,clicks \
  --start-date 2026-03-01 \
  --end-date 2026-03-16 \
  --format csv --output tiktok-report.csv
```

## Meta Routing

When the provider is `meta`, read [references/meta.md](references/meta.md) first. That file is a routing guide — it tells you which specific reference file to load based on the task:

- Auth, config, output → `meta-auth-and-output.md`
- Businesses, ad accounts, campaigns → `meta-accounts-and-objects.md`
- Insights queries and async reports → `meta-reports.md`
- Creatives and activity history → `meta-creative-and-changes.md`
- Pixels, datasets, measurement health → `meta-tracking.md`
- End-to-end recipes → `meta-workflows.md`

Load only the reference file you need. Do not load all of them at once.

## TikTok Routing

When the provider is `tiktok`, read [references/tiktok.md](references/tiktok.md) first. That file is a routing guide — it tells you which specific reference file to load based on the task:

- Auth, config, refresh tokens → `tiktok-auth.md`
- Advertisers, campaigns, ad groups, ads → `tiktok-accounts-and-objects.md`
- Insights queries and async report tasks → `tiktok-reports.md`
- Creative assets, pixels, audiences → `tiktok-creative-and-tracking.md`

Load only the reference file you need. Do not load all of them at once.

## Google Routing

When the provider is `google`, read [references/google.md](references/google.md) first.

## Common Issues

| Problem | What's happening | Fix |
|---------|-----------------|-----|
| "My token expired" | Meta tokens can be short-lived; TikTok tokens expire every 24 hours | Meta: regenerate at [Graph API Explorer](https://developers.facebook.com/tools/explorer/) then `meta auth set`. TikTok: `tiktok auth refresh --app-id ... --app-secret ...` |
| "I don't know my account ID" | You need to discover it first | Meta: `meta businesses list` → `meta ad-accounts list --business-id ...`. Google: `google customers list`. TikTok: `tiktok advertisers list --app-id ... --app-secret ...` |
| "Permission denied" | Token is missing required scopes | Meta: regenerate token with `ads_read` (and `business_management` for discovery). TikTok: check app permissions in TikTok developer portal |
| "Command not found" | CLI not installed or not on PATH | Run `agent-ads --version`. If missing: `npm install -g agent-ads` |
| "doctor says credential store unavailable" | No OS keychain on this machine (common on Linux servers) | Use shell env vars instead: `export META_ADS_ACCESS_TOKEN=...`, `export GOOGLE_ADS_REFRESH_TOKEN=...`, or `export TIKTOK_ADS_ACCESS_TOKEN=...` |
| "TikTok says 'advertiser-id is required'" | Most TikTok commands need an advertiser ID | Add `--advertiser-id <id>`, or set `default_advertiser_id` in `agent-ads.config.json` under `providers.tiktok` |

## Stop Conditions

- Do not drop the provider prefix (`agent-ads meta ...`, not `agent-ads ...`).
- Do not invent cross-provider abstractions (no shared campaign/report/measurement schema).
- Do not reuse Meta auth env vars (`META_ADS_ACCESS_TOKEN`) for future providers.
- Do not guess flag names — use `agent-ads <provider> <command> --help` to confirm.
