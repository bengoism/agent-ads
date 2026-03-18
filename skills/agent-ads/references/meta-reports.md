# Meta Reports

Use this file for `insights` queries, `insights export`, and explicit `report-runs`.

## Sync Insights

```bash
agent-ads meta insights query \
  --account <account-id> \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --since 2026-03-01 \
  --until 2026-03-16
```

Supported knobs:

- `--account` or `--object`
- `--level`
- `--fields` or `--fields-file`
- `--date-preset`
- `--since` and `--until`
- `--time-range-file`
- `--time-increment`
- `--breakdowns`
- `--action-breakdowns`
- `--sort`
- `--filter` or `--filter-file`
- `--attribution-windows`
- `--page-size`
- `--cursor`
- `--all`
- `--max-items`

## High-Level Async Export

```bash
agent-ads meta insights export \
  --account <account-id> \
  --level ad \
  --fields ad_id,ad_name,spend,impressions,actions \
  --async \
  --wait
```

Use this when you want one command to submit, optionally wait, and retrieve results.

## Explicit Report Runs

Submit:

```bash
agent-ads meta report-runs submit \
  --account <account-id> \
  --level ad \
  --fields ad_id,ad_name,spend,impressions,actions
```

Status:

```bash
agent-ads meta report-runs status --id <run-id>
```

Wait:

```bash
agent-ads meta report-runs wait --id <run-id>
```

Results:

```bash
agent-ads meta report-runs results --id <run-id> --all
```

## Gotchas

- `--account` and `--object` are mutually exclusive.
- If you pass `--action-breakdowns`, include `actions` in `--fields`.
- Prefer async export or report runs for large date ranges and heavy breakdowns.
