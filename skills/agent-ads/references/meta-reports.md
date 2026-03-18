# Meta Reports

Use this file for `insights query` (synchronous), `insights export` (high-level async), and `report-runs` (explicit async lifecycle).

## Sync Insights Query

The most common way to pull performance data. Returns results inline.

```bash
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --since 2026-03-01 \
  --until 2026-03-16 \
  --time-increment 1
```

Example output:

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

### Insights flags

#### Object selection (mutually exclusive)

| Flag | Description |
|------|-------------|
| `--account <id>` | Query insights for an ad account (most common) |
| `--object <id>` | Query insights for any Graph API object (campaign, adset, ad, etc.) |

You must use one or the other — not both. If neither is provided and `default_account_id` is set in config, that value is used.

#### Aggregation level

| Flag | Values | Description |
|------|--------|-------------|
| `--level` | `account`, `campaign`, `adset`, `ad` | Granularity of the breakdown |

#### Time range (mutually exclusive groups)

| Flags | Description |
|-------|-------------|
| `--since YYYY-MM-DD --until YYYY-MM-DD` | Explicit date range (both required together) |
| `--date-preset <preset>` | Named preset: `today`, `yesterday`, `last_7d`, `last_14d`, `last_28d`, `last_30d`, `last_90d`, `this_month`, `last_month`, etc. |
| `--time-range-file <path>` | JSON file with `{"since": "...", "until": "..."}` (use `-` for stdin) |

#### Time increment

| Flag | Values | Description |
|------|--------|-------------|
| `--time-increment` | `1` (daily), `7` (weekly), `14` (biweekly), `monthly`, `all_days` | How to bucket the time range into rows |

Most common: `1` for daily, `monthly` for monthly. Omit for a single aggregated row per entity.

#### Breakdowns and dimensions

| Flag | Description |
|------|-------------|
| `--breakdowns <list>` | Comma-separated: `age`, `gender`, `country`, `placement`, `publisher_platform`, `device_platform`, etc. |
| `--action-breakdowns <list>` | Comma-separated: `action_type`, `action_device`, `action_destination`, etc. **Requires `actions` in `--fields`** |
| `--attribution-windows <list>` | Comma-separated: `1d_click`, `7d_click`, `1d_view`, etc. |

#### Filtering and sorting

| Flag | Description |
|------|-------------|
| `--filter <json>` | Inline filter object (repeatable): `'{"field":"impressions","operator":"GREATER_THAN","value":"100"}'` |
| `--filter-file <path>` | JSON file containing a filter array (use `-` for stdin) |
| `--sort <list>` | Comma-separated: `spend_descending`, `impressions_ascending`, etc. |

#### Fields

| Flag | Description |
|------|-------------|
| `--fields <list>` | Comma-separated field names |
| `--fields-file <path>` | Read fields from file (comma or newline separated, `-` for stdin) |

If no fields are specified, defaults to: `account_id`, `account_name`, `campaign_id`, `campaign_name`, `impressions`, `clicks`, `spend`.

#### Pagination

| Flag | Description |
|------|-------------|
| `--page-size <n>` | Items per API page |
| `--cursor <token>` | Resume from cursor |
| `--all` | Auto-paginate all results |
| `--max-items <n>` | Stop after N items |

## High-Level Async Export

For large jobs, use `insights export` with `--async`. This submits an async report run and optionally waits for it to complete and returns the results:

```bash
# Submit and wait in one command
agent-ads meta insights export \
  --account act_1234567890 \
  --level ad \
  --fields ad_id,ad_name,spend,impressions,actions \
  --since 2026-01-01 --until 2026-03-01 \
  --async --wait

# Submit only (returns report_run_id, you poll separately)
agent-ads meta insights export \
  --account act_1234567890 \
  --level ad \
  --fields ad_id,ad_name,spend \
  --async
```

Additional flags for async mode:

| Flag | Default | Description |
|------|---------|-------------|
| `--async` | off | Use async report run instead of inline query |
| `--wait` | off | Poll until complete, then return results (requires `--async`) |
| `--poll-interval-seconds` | `5` | Seconds between status checks |
| `--wait-timeout-seconds` | `3600` | Maximum wait time before timing out |

Without `--async`, `insights export` behaves identically to `insights query`.

## Explicit Report Runs

For full control over the async lifecycle:

### Submit

```bash
agent-ads meta report-runs submit \
  --account act_1234567890 \
  --level ad \
  --fields ad_id,ad_name,spend,impressions,actions \
  --since 2026-03-01 --until 2026-03-16
```

Returns: `{ "report_run_id": "12345", "id": "12345", ... }`

### Check status

```bash
agent-ads meta report-runs status --id 12345
```

Returns: `{ "id": "12345", "async_status": "Job Running", "async_percent_completion": 45, ... }`

### Wait for completion

```bash
agent-ads meta report-runs wait --id 12345
agent-ads meta report-runs wait --id 12345 --poll-interval-seconds 10 --wait-timeout-seconds 1800
```

Polls until `async_status` contains "complete" or `async_percent_completion` reaches 100. Returns the final status object.

### Fetch results

```bash
agent-ads meta report-runs results --id 12345 --all
agent-ads meta report-runs results --id 12345 --all --fields ad_id,ad_name,spend --format csv --output results.csv
```

## Gotchas

- `--account` and `--object` are mutually exclusive. Use `--account` for ad-account-level queries, `--object` for querying a specific campaign/adset/ad.
- If you use `--action-breakdowns`, you **must** include `actions` in `--fields`. The CLI validates this and will error if `actions` is missing.
- Prefer async (`--async --wait` or explicit `report-runs`) for large date ranges, ad-level queries across many ads, or heavy breakdown combinations.
- `--time-increment` and `--date-preset` are separate concepts: `--date-preset` sets the time range, `--time-increment` sets how rows are bucketed within that range.
