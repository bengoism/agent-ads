# TikTok Guide

This is the routing guide for the TikTok provider. Read this first when the user wants to work with TikTok Ads. Then load only the specific reference file linked below.

## Route to the Right Reference

| Task | First command | Reference file |
|------|---------------|----------------|
| Set up auth, check config, refresh tokens | `agent-ads tiktok doctor` | [tiktok-auth.md](tiktok-auth.md) |
| List advertisers and account info | `agent-ads tiktok advertisers list` | [tiktok-accounts-and-objects.md](tiktok-accounts-and-objects.md) |
| List campaigns, ad groups, or ads | `agent-ads tiktok campaigns list --advertiser-id <id>` | [tiktok-accounts-and-objects.md](tiktok-accounts-and-objects.md) |
| Run a performance report (sync) | `agent-ads tiktok insights query ...` | [tiktok-reports.md](tiktok-reports.md) |
| Manage async report tasks | `agent-ads tiktok report-runs submit/status/cancel` | [tiktok-reports.md](tiktok-reports.md) |
| Search video/image creative assets | `agent-ads tiktok creatives videos --advertiser-id <id>` | [tiktok-creative-and-tracking.md](tiktok-creative-and-tracking.md) |
| List pixels or audiences | `agent-ads tiktok pixels list --advertiser-id <id>` | [tiktok-creative-and-tracking.md](tiktok-creative-and-tracking.md) |

Load only the reference file you need. Do not preload all of them.

## Quick Reference

### Auth (secure storage or shell env — never from flags or config files)

| Variable | Required | Purpose |
|----------|----------|---------|
| `TIKTOK_ADS_ACCESS_TOKEN` | No | Shell override / CI fallback access token |
| `TIKTOK_ADS_REFRESH_TOKEN` | No | Refresh token (for `auth refresh` flow) |
| `TIKTOK_ADS_APP_ID` | For `advertisers list` and `auth refresh` | TikTok app ID |
| `TIKTOK_ADS_APP_SECRET` | For `advertisers list` and `auth refresh` | TikTok app secret |

Persistent local auth is stored with `agent-ads tiktok auth set`.

### Config precedence

Token precedence: shell env > OS credential store

Refresh token precedence for `auth refresh`: `TIKTOK_ADS_REFRESH_TOKEN` > OS credential store

Non-secret precedence: CLI flags > shell env > `agent-ads.config.json` (under `providers.tiktok`)

### Output defaults

- stdout: data-only JSON (no wrapper)
- stderr: errors as JSON, warnings as text
- `--envelope` adds metadata/paging wrapper
- `--format json|jsonl|csv`

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Transport or internal failure |
| 2 | Config or argument failure |
| 4 | TikTok API failure |

### Pagination (different from Meta)

TikTok uses page-number pagination, not cursor-based:

| Flag | What it does |
|------|-------------|
| `--page <n>` | Page number (1-indexed) |
| `--page-size <n>` | Items per page |
| `--all` | Auto-follow all pages |
| `--max-items <n>` | Stop after N items |

## Common Mistakes

- Forgetting `TIKTOK_ADS_ACCESS_TOKEN` before running API commands
- Not passing `--advertiser-id` and not setting `TIKTOK_ADS_DEFAULT_ADVERTISER_ID`
- Using `--cursor` (that's Meta) instead of `--page` (TikTok)
- TikTok tokens expire every 24 hours — use `agent-ads tiktok auth refresh` to rotate
- Not providing `--app-id` / `--app-secret` for `advertisers list` (the OAuth endpoint requires them)
