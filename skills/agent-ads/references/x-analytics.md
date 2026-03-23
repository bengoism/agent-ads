# X Analytics

Use this guide when the task is performance reporting, active entity syncs, or async analytics jobs in X Ads.

## Synchronous Analytics

```bash
agent-ads x analytics query \
  --account-id 18ce54d4x5t \
  --entity campaign \
  --entity-id c123 \
  --start-time 2026-03-01T00:00:00Z \
  --end-time 2026-03-07T00:00:00Z \
  --granularity day \
  --placement all-on-twitter \
  --metric-group engagement,billing
```

Rules:

- max 7-day window
- 1 to 20 `--entity-id` values
- RFC 3339 timestamps
- use whole-hour timestamps for consistency with the wider analytics surface

## Reach And Frequency

```bash
agent-ads x analytics reach \
  --account-id 18ce54d4x5t \
  --level campaigns \
  --id c123 \
  --start-time 2026-03-01T00:00:00Z \
  --end-time 2026-03-07T00:00:00Z
```

Use this instead of `analytics query` when the task is reach or average frequency rather than standard stats.

## Active Entities

```bash
agent-ads x analytics active-entities \
  --account-id 18ce54d4x5t \
  --entity line-item \
  --start-time 2026-03-01T00:00:00Z \
  --end-time 2026-03-07T00:00:00Z
```

This is first-class because X recommends it for efficient analytics syncs before pulling full stats.

Rules:

- timestamps must be RFC 3339 and aligned to whole hours
- use campaign/funding-instrument/line-item filters when narrowing sync scope

## Async Analytics Jobs

```bash
agent-ads x analytics jobs submit \
  --account-id 18ce54d4x5t \
  --entity campaign \
  --entity-id c123 \
  --start-time 2026-01-01T00:00:00Z \
  --end-time 2026-03-01T00:00:00Z \
  --granularity day \
  --placement all-on-twitter \
  --metric-group engagement,billing

agent-ads x analytics jobs status --account-id 18ce54d4x5t --job-id job123
agent-ads x analytics jobs wait --account-id 18ce54d4x5t --job-id job123
agent-ads x analytics jobs download --account-id 18ce54d4x5t --job-id job123 --wait
```

Rules:

- max 90-day window
- max 20 entity IDs
- at most one `--segmentation-type`
- `download` follows the normal stdout, `--format`, and `--output` pipeline after resolving the job URL
