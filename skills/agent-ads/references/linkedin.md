# LinkedIn Guide

This is the provider guide for LinkedIn Marketing API work in `agent-ads`.

## Start Here

| Task | First command |
|------|---------------|
| Set up auth or inspect config | `agent-ads linkedin doctor` |
| List accessible ad accounts | `agent-ads linkedin ad-accounts list` |
| Search or fetch one ad account | `agent-ads linkedin ad-accounts search` / `get --account-id <id>` |
| Explore campaign groups or campaigns | `agent-ads linkedin campaign-groups list --account-id <id>` / `campaigns list --account-id <id>` |
| Inspect creatives | `agent-ads linkedin creatives list --account-id <id>` |
| Run reporting | `agent-ads linkedin analytics query --finder ... --account-id <id> ...` |

## Auth Model

LinkedIn v1 uses one credential:

| Credential | Persistent storage | Shell override |
|------------|--------------------|----------------|
| Access token | `agent-ads linkedin auth set` | `LINKEDIN_ADS_ACCESS_TOKEN` |

Optional defaults:

| Variable / config | Purpose |
|-------------------|---------|
| `LINKEDIN_ADS_DEFAULT_ACCOUNT_ID` / `providers.linkedin.default_account_id` | Default ad account for scoped commands |
| `LINKEDIN_ADS_API_VERSION` / `providers.linkedin.api_version` | Override the `Linkedin-Version` header (default `202603`) |

## Command Model

- `ad-accounts list` discovers accessible accounts through the authenticated-user account-access surface and adds `authenticated_user_role` to each hydrated account object.
- `ad-accounts search` stays account-native and returns raw LinkedIn account search results without role enrichment.
- `campaign-groups list`, `campaigns list|get`, and `creatives list|get` stay provider-native and map directly to LinkedIn Marketing API resources.
- `analytics query` wraps LinkedIn `adAnalytics` finder queries.
- Keep LinkedIn finder semantics intact. Do not remap them into Meta/TikTok `insights query`.

## ID Rules

- `--account-id`, `--campaign-group-id`, and `--campaign-id` accept either raw numeric IDs or full URNs.
- `--creative-id` accepts a numeric ID or a sponsored creative URN and is normalized to a URN internally.
- Search and analytics filters use URNs where LinkedIn expects them.

## Pagination

LinkedIn list commands use cursor pagination:

| Flag | Meaning |
|------|---------|
| `--page-token <token>` | Resume from `metadata.nextPageToken` |
| `--page-size <n>` | Items per API request |
| `--all` | Follow all pages |
| `--max-items <n>` | Stop after N items |

## Reporting Finder Rules

- `--finder analytics`: exactly one `--pivot`, requires `--time-granularity`
- `--finder statistics`: one to three `--pivot` values, requires `--time-granularity`
- `--finder attributed-revenue-metrics`: one to three pivots, only `ACCOUNT`, `CAMPAIGN_GROUP`, `CAMPAIGN`, requires both `--since` and `--until`, 30-366 day range, within the last year

## Common Mistakes

- Forgetting `--account-id` on scoped commands without setting a default
- Passing `--creative-id` to `--finder attributed-revenue-metrics`
- Mixing raw IDs and URNs inconsistently in shell scripts instead of letting the CLI normalize them
- Expecting write operations; LinkedIn support in `agent-ads` is read-only
