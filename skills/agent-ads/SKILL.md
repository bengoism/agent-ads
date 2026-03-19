---
name: agent-ads
description: >
  Operate the provider-first ads CLI from the terminal. Use when the user wants
  to inspect available ad providers, route work into the implemented Meta,
  Google, TikTok, or Pinterest providers, or keep ad-provider commands
  explicit inside `agent-ads`.
---

# Agent Ads

`agent-ads` is a Unix-first CLI for querying ad platform APIs. It supports Meta (Facebook/Instagram) Marketing API, Google Ads, TikTok Business API, and Pinterest Ads API. All commands are read-only.

This file is the only public skill entrypoint. It routes you to provider-specific reference docs.

## What You Can Ask

Plain-English prompts that work. The agent translates these into the right CLI commands.

**Performance & reporting**
- "Show me my Meta ad spend by campaign for the last 7 days"
- "Run this GAQL query against Google Ads"
- "What's my TikTok campaign performance this month?"
- "What did my Pinterest campaigns spend last week?"
- "Pull a daily breakdown of impressions and clicks across all campaigns"
- "Export last week's Meta performance report to CSV"
- "Run an async report for all my TikTok ads since January"
- "Submit a Pinterest report run and wait for the result URL"

**Accounts & campaigns**
- "What businesses do I have access to in Meta?"
- "List all my accessible Google Ads customers"
- "List all my TikTok advertisers"
- "List my Pinterest ad accounts"
- "Show me the campaigns in my Meta ad account"
- "Which ad sets are running right now?"

**Creative & tracking**
- "Show my TikTok video creatives"
- "Check if my Meta pixel is working"
- "Find broken pixels across my Meta accounts"
- "What audiences do I have in TikTok?"
- "What audiences do I have in Pinterest?"

**Setup & troubleshooting**
- "Help me set up my Meta auth token"
- "Is my TikTok token still valid?"
- "Refresh my Pinterest token"
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
| ad account | Pinterest | `--ad-account-id 123...` flag |
| campaign performance / report | Meta/TikTok | `insights query` command |
| analytics / report | Pinterest | `analytics query`, `report-runs ...` |
| GAQL / query | Google | `gaql search`, `gaql search-stream` |
| pixel / tag | Meta | `pixels list`, `pixel-health get` |
| pixel | TikTok | `pixels list` |
| audience / custom audience | TikTok | `audiences list` |
| audience / custom audience | Pinterest | `audiences list`, `audiences get` |
| creative / ad content | Meta | `creatives get`, `creatives preview` |
| creative / video / image | TikTok | `creatives videos`, `creatives images` |
| conversion tracking | Meta | `custom-conversions list`, `datasets get` |
| token / access token | all | stored via `auth set` or shell env var |

## Command Syntax Rules

Every command starts with `agent-ads <provider>`. This is intentional — each ad platform has different APIs, auth, object models, and semantics, so the CLI keeps them separate rather than papering over differences.

- Canonical: `agent-ads meta insights query ...`
- Never: `agent-ads insights query ...` (no implicit provider)
- Never: `agent-ads meta:insights:query` (no colon syntax)

## Provider Routing

| Provider | Status | First command | Reference guide |
|----------|--------|---------------|-----------------|
| `meta` | Implemented | `agent-ads meta --help` | [references/meta.md](references/meta.md) |
| `google` | Implemented | `agent-ads google --help` | [references/google.md](references/google.md) |
| `tiktok` | Implemented | `agent-ads tiktok --help` | [references/tiktok.md](references/tiktok.md) |
| `pinterest` | Implemented | `agent-ads pinterest --help` | [references/pinterest.md](references/pinterest.md) |

Check live: `agent-ads providers list`

Each routing guide tells you which specific reference file to load based on the task. Load only the file you need — do not preload all of them.

## Shared Behavior

These apply to every provider and every command.

### Output

- **stdout**: data-only JSON by default (just the array or object, no wrapper)
- **stderr**: errors as JSON, warnings as plain text
- **`--envelope`**: wraps stdout with `{ "data": ..., "meta": ..., "paging": ... }`
- **Formats**: `--format json|jsonl|csv`, `--output <path>`, `--pretty`

### Config precedence

- Secrets: shell env > OS credential store. Never from flags or config files.
- Non-secrets: CLI flags > shell env > `agent-ads.config.json`
- Persistent auth: `agent-ads <provider> auth set`
- Shell override / CI: provider-specific env vars (e.g. `META_ADS_ACCESS_TOKEN`, `GOOGLE_ADS_REFRESH_TOKEN`, `TIKTOK_ADS_ACCESS_TOKEN`, `PINTEREST_ADS_REFRESH_TOKEN`)
- Linux secure storage requires a running Secret Service provider (GNOME Keyring, KWallet)

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

### Global flags

| Flag | Default | What it does |
|------|---------|-------------|
| `--config <path>` | `agent-ads.config.json` | Config file path |
| `--format json\|jsonl\|csv` | `json` | Output format |
| `--output <path>` | stdout | Write to file (`-` for stdout) |
| `--pretty` | off | Pretty-print JSON |
| `--envelope` | off | Wrap data with metadata, paging, warnings |
| `--include-meta` | off | Add metadata columns in CSV mode |
| `--api-version <v>` | provider default | Provider API version override |
| `--timeout-seconds <n>` | `60` | HTTP request timeout |
| `-q` / `-v` | warn | Quiet mode or verbose logging (`-vv` for debug) |

Pagination flags differ by provider — see each provider's routing guide for details.

## Common Issues

| Problem | What's happening | Fix |
|---------|-----------------|-----|
| "My token expired" | Meta tokens can be short-lived; TikTok tokens expire every 24 hours; Pinterest relies on OAuth refresh tokens | Meta: regenerate at [Graph API Explorer](https://developers.facebook.com/tools/explorer/) then `meta auth set`. TikTok: `tiktok auth refresh --app-id ... --app-secret ...`. Pinterest: `pinterest auth refresh` |
| "I don't know my account ID" | You need to discover it first | Meta: `meta businesses list` then `meta ad-accounts list --business-id ...`. Google: `google customers list`. TikTok: `tiktok advertisers list --app-id ... --app-secret ...`. Pinterest: `pinterest ad-accounts list` |
| "Permission denied" | Token is missing required scopes | Meta: regenerate token with `ads_read` (and `business_management` for discovery). TikTok: check app permissions in TikTok developer portal. Pinterest: regenerate app credentials/tokens with the required ads-read scope in the Pinterest developer portal |
| "Command not found" | CLI not installed or not on PATH | Run `agent-ads --version`. If missing: `npm install -g agent-ads` |
| "doctor says credential store unavailable" | No OS keychain on this machine (common on Linux servers) | Use shell env vars instead: `export META_ADS_ACCESS_TOKEN=...`, `export GOOGLE_ADS_REFRESH_TOKEN=...`, `export TIKTOK_ADS_ACCESS_TOKEN=...`, or `export PINTEREST_ADS_REFRESH_TOKEN=...` |
| "TikTok says 'advertiser-id is required'" | Most TikTok commands need an advertiser ID | Add `--advertiser-id <id>`, or set `default_advertiser_id` in config under `providers.tiktok` |
| "Pinterest says 'ad account ID is required'" | Most Pinterest commands are scoped to an ad account | Add `--ad-account-id <id>`, or set `default_ad_account_id` in config under `providers.pinterest` |

## Stop Conditions

- Do not drop the provider prefix (`agent-ads meta ...`, not `agent-ads ...`).
- Do not invent cross-provider abstractions (no shared campaign/report/measurement schema).
- Do not reuse Meta auth env vars (`META_ADS_ACCESS_TOKEN`) for other providers.
- Do not guess flag names — use `agent-ads <provider> <command> --help` to confirm.
