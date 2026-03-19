# Google Guide

This is the provider guide for Google Ads. Read this first when the user wants to work with Google Ads through `agent-ads`.

## Start Here

| Task | First command |
|------|---------------|
| Set up auth or inspect config | `agent-ads google doctor` |
| List accessible customers | `agent-ads google customers list` |
| Explore a customer hierarchy | `agent-ads google customers hierarchy --customer-id <id>` |
| List campaigns, ad groups, or ads | `agent-ads google campaigns list --customer-id <id>` |
| Run a provider-native query | `agent-ads google gaql search --customer-id <id> --query "..."` |
| Stream a large query result | `agent-ads google gaql search-stream --customer-id <id> --query-file query.sql` |

## Auth Model

Google Ads uses four credential pieces:

| Credential | Persistent storage | Shell override |
|------------|--------------------|----------------|
| Developer token | `agent-ads google auth set` | `GOOGLE_ADS_DEVELOPER_TOKEN` |
| OAuth client ID | `agent-ads google auth set` | `GOOGLE_ADS_CLIENT_ID` |
| OAuth client secret | `agent-ads google auth set` | `GOOGLE_ADS_CLIENT_SECRET` |
| OAuth refresh token | `agent-ads google auth set` | `GOOGLE_ADS_REFRESH_TOKEN` |

`agent-ads` exchanges the refresh token for an access token on demand. It does not persist access tokens.

Optional defaults:

| Variable / config | Purpose |
|-------------------|---------|
| `GOOGLE_ADS_DEFAULT_CUSTOMER_ID` / `providers.google.default_customer_id` | Default customer for scoped commands |
| `GOOGLE_ADS_LOGIN_CUSTOMER_ID` / `providers.google.login_customer_id` | Manager account header for scoped requests |

## Command Model

- `customers list` is discovery: it lists directly accessible customer resource names.
- `customers hierarchy` is a convenience GAQL wrapper over `customer_client`.
- `campaigns list`, `adgroups list`, and `ads list` are convenience GAQL wrappers with curated default fields.
- `gaql search` and `gaql search-stream` are the native Google escape hatches for anything more complex.
- Keep Google commands provider-native. Do not remap them into `insights query`.

## Pagination

Google uses page-token pagination:

| Flag | Meaning |
|------|---------|
| `--page-token <token>` | Resume from `nextPageToken` |
| `--all` | Follow all pages |
| `--max-items <n>` | Stop after N rows |

Google `search` returns fixed-size API pages, so there is no page-size override.

`gaql search-stream` does not use page tokens; use `--max-items` if needed.

## Common Mistakes

- Forgetting one of the four required Google credential pieces
- Expecting a cross-provider `insights query` surface; for Google, use `gaql search`
- Omitting `login_customer_id` when querying through a manager account
