# Meta Accounts And Objects

Use this file for business discovery, ad-account listing, and campaign/adset/ad metadata.

## Businesses

List all businesses accessible to your token:

```bash
agent-ads meta businesses list
```

With custom fields and full pagination:

```bash
agent-ads meta businesses list --all --fields id,name,verification_status
```

Default fields: `id`, `name`, `verification_status`.

## Ad Accounts

List ad accounts under a business. The `--scope` flag controls which relationship to query:

| Scope | Description |
|-------|-------------|
| `accessible` (default) | All ad accounts the business can access |
| `owned` | Only ad accounts owned by the business |
| `pending-client` | Ad accounts with pending client relationships |

```bash
# Default scope (accessible)
agent-ads meta ad-accounts list --business-id 1234567890

# Explicit scope
agent-ads meta ad-accounts list --business-id 1234567890 --scope owned

# With custom fields
agent-ads meta ad-accounts list --business-id 1234567890 --fields id,name,account_status,currency,timezone_name
```

Default fields: `id`, `account_id`, `name`, `account_status`, `currency`, `timezone_name`.

If `default_business_id` is set in config, you can omit `--business-id`.

## Campaigns

```bash
agent-ads meta campaigns list --account act_1234567890
agent-ads meta campaigns list --account act_1234567890 --all --fields id,name,status,objective
```

Default fields: `id`, `name`, `status`, `effective_status`, `objective`, `created_time`, `updated_time`.

## Ad Sets

```bash
agent-ads meta adsets list --account act_1234567890
agent-ads meta adsets list --account act_1234567890 --all --fields id,name,campaign_id,status,daily_budget
```

Default fields: `id`, `name`, `campaign_id`, `status`, `effective_status`, `daily_budget`, `lifetime_budget`, `billing_event`.

## Ads

```bash
agent-ads meta ads list --account act_1234567890
agent-ads meta ads list --account act_1234567890 --all --fields id,name,adset_id,status
```

Default fields: `id`, `name`, `adset_id`, `campaign_id`, `status`, `effective_status`, `creative{id,name}`.

## Shared Flags

All object list commands accept:

| Flag | Description |
|------|-------------|
| `--account <id>` | Ad account ID (omit if `default_account_id` is set in config) |
| `--fields <list>` | Comma-separated field names |
| `--fields-file <path>` | Read field names from a file (comma or newline separated, `-` for stdin) |
| `--page-size <n>` | Items per API request |
| `--cursor <token>` | Resume from a cursor |
| `--all` | Auto-paginate through all results |
| `--max-items <n>` | Stop after collecting N items |

## Account ID Normalization

Account IDs accept either raw numeric (`1234567890`) or prefixed (`act_1234567890`) format. The CLI always normalizes to `act_<id>` before making API calls. Output also uses the `act_` prefix.
