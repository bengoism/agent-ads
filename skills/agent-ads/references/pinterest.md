# Pinterest Guide

This is the provider guide for Pinterest Ads. Read this first when the user wants to work with Pinterest through `agent-ads`.

## Start Here

| Task | First command |
|------|---------------|
| Set up auth or inspect config | `agent-ads pinterest doctor` |
| List accessible ad accounts | `agent-ads pinterest ad-accounts list` |
| Explore campaigns, ad groups, or ads | `agent-ads pinterest campaigns list --ad-account-id <id>` |
| Run synchronous analytics | `agent-ads pinterest analytics query --ad-account-id <id> ...` |
| Run async reporting | `agent-ads pinterest report-runs submit --ad-account-id <id> ...` |
| Inspect audiences | `agent-ads pinterest audiences list --ad-account-id <id>` |
| Break down targeting performance | `agent-ads pinterest targeting-analytics query --ad-account-id <id> ...` |

## Auth Model

Pinterest uses four credential pieces:

| Credential | Persistent storage | Shell override |
|------------|--------------------|----------------|
| App ID | `agent-ads pinterest auth set` | `PINTEREST_ADS_APP_ID` |
| App secret | `agent-ads pinterest auth set` | `PINTEREST_ADS_APP_SECRET` |
| Access token | `agent-ads pinterest auth set` | `PINTEREST_ADS_ACCESS_TOKEN` |
| Refresh token | `agent-ads pinterest auth set` | `PINTEREST_ADS_REFRESH_TOKEN` |

`agent-ads pinterest auth refresh` exchanges the refresh token and updates the stored access token (and rotated refresh token, if Pinterest returns one).

Optional defaults:

| Variable / config | Purpose |
|-------------------|---------|
| `PINTEREST_ADS_DEFAULT_AD_ACCOUNT_ID` / `providers.pinterest.default_ad_account_id` | Default ad account for scoped commands |

## Command Model

- `ad-accounts list|get` is account discovery.
- `campaigns list`, `adgroups list`, and `ads list` stay provider-native and use Pinterest bookmark pagination.
- `analytics query` wraps the synchronous analytics endpoints for `ad_account`, `campaign`, `ad_group`, `ad`, and `ad_pin`.
- `report-runs submit|status|wait` wraps Pinterest async report creation and polling.
- `audiences list|get` and `targeting-analytics query` cover the read-only audience and targeting surfaces included in v1.
- Keep Pinterest commands provider-native. Do not remap them into Meta or TikTok naming.

## Pagination

Pinterest list commands use bookmark pagination:

| Flag | Meaning |
|------|---------|
| `--bookmark <token>` | Resume from a Pinterest bookmark |
| `--page-size <n>` | Items per API request |
| `--all` | Follow all pages |
| `--max-items <n>` | Stop after N items |

## Common Mistakes

- Forgetting to set both app credentials and both tokens
- Omitting `--ad-account-id` on scoped commands without setting a default
- Using `analytics query` for workloads that are better handled by `report-runs submit|wait`
- Expecting write operations; Pinterest support in `agent-ads` is read-only
