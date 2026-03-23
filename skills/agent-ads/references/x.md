# X Guide

This is the provider guide for X Ads in `agent-ads`.

## Start Here

| Task | First command |
|------|---------------|
| Set up auth or inspect config | `agent-ads x doctor` |
| List accessible ads accounts | `agent-ads x accounts list` |
| Inspect the authenticated user's role in one account | `agent-ads x authenticated-user-access get --account-id <id>` |
| Explore campaigns and line items | `agent-ads x campaigns list --account-id <id>` / `line-items list --account-id <id>` |
| Inspect creatives and promoted tweets | `agent-ads x promoted-tweets list --account-id <id>` |
| Run analytics | `agent-ads x analytics query --account-id <id> ...` |

## Auth Model

X uses four OAuth 1.0a credential pieces. There is no browser-assisted auth flow in v1.

| Credential | Persistent storage | Shell override |
|------------|--------------------|----------------|
| Consumer key | `agent-ads x auth set` | `X_ADS_CONSUMER_KEY` |
| Consumer secret | `agent-ads x auth set` | `X_ADS_CONSUMER_SECRET` |
| Access token | `agent-ads x auth set` | `X_ADS_ACCESS_TOKEN` |
| Access token secret | `agent-ads x auth set` | `X_ADS_ACCESS_TOKEN_SECRET` |

Optional defaults:

| Variable / config | Purpose |
|-------------------|---------|
| `X_ADS_DEFAULT_ACCOUNT_ID` / `providers.x.default_account_id` | Default ads account for scoped commands |
| `X_ADS_API_BASE_URL` / `providers.x.api_base_url` | Override the Ads API base URL (default `https://ads-api.x.com`) |
| `X_ADS_API_VERSION` / `providers.x.api_version` | Override the Ads API version (default `12`) |

## Command Model

- `accounts list|get` and `authenticated-user-access get` cover account discovery and account access inspection.
- Campaign-management nouns stay provider-native: `campaigns`, `line-items`, `funding-instruments`, `promotable-users`, `promoted-accounts`, and `targeting-criteria`.
- Creative/media nouns also stay provider-native: `promoted-tweets`, `draft-tweets`, `scheduled-tweets`, `cards`, `account-media`, `media-library`, `account-apps`, and `scoped-timeline`.
- Audience and measurement surfaces are separate command families: `custom-audiences`, `do-not-reach-lists`, `web-event-tags`, `app-lists`, and `ab-tests`.
- Analytics is split into `analytics query`, `analytics reach`, `analytics active-entities`, and `analytics jobs ...`.
- The CLI is read-only. The only POST-based exception is async analytics job submission.

## Pagination

X list commands use cursor pagination:

| Flag | Meaning |
|------|---------|
| `--cursor <token>` | Resume from an X cursor |
| `--page-size <n>` | Items per API request (`count`) |
| `--all` | Follow all available pages |
| `--max-items <n>` | Stop after N total items |

## Analytics Rules

- Synchronous analytics queries support at most a 7-day window.
- Async analytics jobs support at most a 90-day window.
- Analytics job submissions accept at most 20 entity IDs.
- `analytics active-entities` requires whole-hour RFC 3339 timestamps.
- Async analytics accepts at most one segmentation type per request.

## Common Mistakes

- Forgetting one of the four OAuth 1.0a secrets
- Omitting `--account-id` on scoped commands without setting `providers.x.default_account_id`
- Using non-hour timestamps like `2026-03-01T12:34:56Z` for analytics
- Expecting write operations for campaigns, tweets, or audiences; X support in `agent-ads` is read-only
