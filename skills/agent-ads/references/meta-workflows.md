# Meta Workflows

Use this file when the user wants an end-to-end recipe instead of a command-family reference.

## Multi-Client Discovery

```bash
agent-ads meta businesses list
agent-ads meta ad-accounts list --business-id <business-id> --scope accessible
```

## Daily Performance Export

```bash
agent-ads meta insights query \
  --account <account-id> \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --time-increment 1 \
  --since 2026-03-01 \
  --until 2026-03-16 \
  --format csv \
  --output campaign-daily.csv
```

## Async Export For Large Jobs

```bash
agent-ads meta insights export \
  --account <account-id> \
  --level ad \
  --fields ad_id,ad_name,spend,impressions,actions \
  --async \
  --wait \
  --output results.json
```

If you need explicit control:

```bash
agent-ads meta report-runs submit --account <account-id> --level ad --fields ad_id,ad_name,spend,impressions,actions
agent-ads meta report-runs wait --id <run-id>
agent-ads meta report-runs results --id <run-id> --all
```

## Forensic Diagnosis

```bash
agent-ads meta activities list --account <account-id> --since 2026-03-10T00:00:00Z --all
agent-ads meta creatives preview --ad <ad-id>
agent-ads meta pixel-health get --pixel <pixel-id>
```
