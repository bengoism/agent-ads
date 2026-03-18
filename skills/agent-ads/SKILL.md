---
name: agent-ads
description: >
  Operate the provider-first ads CLI from the terminal. Use when the user wants
  to inspect available ad providers, route work into the implemented Meta
  provider, or keep ad-provider commands explicit inside `agent-ads`.
---

# Agent Ads

`agent-ads` is a Unix-first CLI for querying ad platform APIs. It supports Meta (Facebook/Instagram) Marketing API today, with Google and TikTok namespaces reserved for future implementation. All commands are read-only.

This file is the only public skill entrypoint. It routes you to provider-specific reference docs.

## Command Syntax Rules

Every command starts with `agent-ads <provider>`. This is intentional — each ad platform has different APIs, auth, object models, and semantics, so the CLI keeps them separate rather than papering over differences.

- Canonical: `agent-ads meta insights query ...`
- Never: `agent-ads insights query ...` (no implicit provider)
- Never: `agent-ads meta:insights:query` (no colon syntax)

## Provider Status

| Provider | Status | Summary |
|----------|--------|---------|
| `meta` | Implemented | Read-only Meta Marketing API: accounts, campaigns, insights, creatives, tracking |
| `google` | Reserved | Namespace exists, commands not implemented |
| `tiktok` | Reserved | Namespace exists, commands not implemented |

Check live: `agent-ads providers list`

## Start Here

| Need | First command | Then read |
|------|---------------|-----------|
| See which providers exist | `agent-ads providers list` | this file |
| Work with Meta (accounts, reports, creatives, tracking) | `agent-ads meta --help` | [references/meta.md](references/meta.md) |
| Work with Google or TikTok | n/a | tell the user the namespace exists but is not implemented yet |

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
| `--api-version <v>` | `v25.0` | Meta API version override |
| `--timeout-seconds <n>` | `60` | HTTP request timeout |
| `-q` / `-v` | warn | Quiet mode or verbose logging (`-vv` for debug) |

## Pagination Flags

All list commands support:

| Flag | What it does |
|------|-------------|
| `--page-size <n>` | Items per API request |
| `--cursor <token>` | Resume from cursor |
| `--all` | Auto-follow all pages |
| `--max-items <n>` | Stop after N items |

## Output

- **stdout**: data-only JSON by default (just the array or object, no wrapper)
- **stderr**: errors as JSON, warnings as plain text
- **`--envelope`**: wraps stdout with `{ "data": ..., "meta": ..., "paging": ... }`
- **Exit codes**: 0 = success, 1 = transport/internal, 2 = config/argument, 3 = API error

## Token Permissions

The user's `META_ADS_ACCESS_TOKEN` needs specific Meta permissions (scopes) depending on what commands they run:

| Permission | Needed for |
|------------|------------|
| `ads_read` | All `--account` commands: campaigns, insights, creatives, pixels |
| `business_management` | `businesses list` and `ad-accounts list` (discovery) |

Both are read-only — no write access is granted. If the user gets a "Missing Permission" error, they need to regenerate their token with the correct scopes at the [Graph API Explorer](https://developers.facebook.com/tools/explorer/).

## Auth Storage

- Persistent auth: `agent-ads meta auth set`
- Shell override / CI fallback: `META_ADS_ACCESS_TOKEN=... agent-ads ...`
- Linux secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet

## Worked Example

A typical multi-step session discovering accounts and pulling a report:

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

## Meta Routing

When the provider is `meta`, read [references/meta.md](references/meta.md) first. That file is a routing guide — it tells you which specific reference file to load based on the task:

- Auth, config, output → `meta-auth-and-output.md`
- Businesses, ad accounts, campaigns → `meta-accounts-and-objects.md`
- Insights queries and async reports → `meta-reports.md`
- Creatives and activity history → `meta-creative-and-changes.md`
- Pixels, datasets, measurement health → `meta-tracking.md`
- End-to-end recipes → `meta-workflows.md`

Load only the reference file you need. Do not load all of them at once.

## Stop Conditions

- Do not drop the provider prefix (`agent-ads meta ...`, not `agent-ads ...`).
- Do not invent cross-provider abstractions (no shared campaign/report/measurement schema).
- Do not reuse Meta auth env vars (`META_ADS_ACCESS_TOKEN`) for future providers.
- Do not guess flag names — use `agent-ads meta <command> --help` to confirm.
