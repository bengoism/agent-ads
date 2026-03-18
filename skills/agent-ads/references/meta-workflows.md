# Meta Workflows

Use this file for end-to-end recipes. Each workflow is a sequence of commands you can run in order.

## 1. Multi-Client Account Discovery

Start from your token and discover all businesses and their ad accounts:

```bash
# Step 1: List all businesses
agent-ads meta businesses list --all --pretty

# Step 2: For each business, list accessible ad accounts
agent-ads meta ad-accounts list --business-id 1234567890 --scope accessible --all

# Step 3: Optionally check which accounts you own vs. have access to
agent-ads meta ad-accounts list --business-id 1234567890 --scope owned --all
```

## 2. Daily Performance Report

Pull daily campaign performance for a date range, exported as CSV:

```bash
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend,cpc,ctr \
  --since 2026-03-01 \
  --until 2026-03-16 \
  --time-increment 1 \
  --format csv \
  --output campaign-daily.csv
```

For the last 7 days with a named preset:

```bash
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --date-preset last_7d \
  --time-increment 1 \
  --all
```

## 3. Ad-Level Report With Breakdowns

Detailed ad-level performance with age and gender breakdowns:

```bash
agent-ads meta insights query \
  --account act_1234567890 \
  --level ad \
  --fields ad_id,ad_name,impressions,clicks,spend,actions \
  --date-preset last_30d \
  --breakdowns age,gender \
  --action-breakdowns action_type \
  --all
```

Note: `actions` must be in `--fields` when using `--action-breakdowns`.

## 4. Large Async Export

For reports that are too large for sync queries (many ads, long date ranges, heavy breakdowns):

### One-command approach

```bash
agent-ads meta insights export \
  --account act_1234567890 \
  --level ad \
  --fields ad_id,ad_name,spend,impressions,clicks,actions \
  --since 2026-01-01 --until 2026-03-01 \
  --async --wait \
  --format csv \
  --output large-report.csv
```

### Step-by-step approach (explicit control)

```bash
# Submit the job
agent-ads meta report-runs submit \
  --account act_1234567890 \
  --level ad \
  --fields ad_id,ad_name,spend,impressions,clicks \
  --since 2026-01-01 --until 2026-03-01

# Note the report_run_id from the output, then wait
agent-ads meta report-runs wait --id 12345678

# Fetch results
agent-ads meta report-runs results --id 12345678 --all --format csv --output results.csv
```

## 5. Forensic Diagnosis

Investigate what changed, inspect the creative, and check pixel health:

```bash
# Step 1: What changed in the account recently?
agent-ads meta activities list \
  --account act_1234567890 \
  --since 2026-03-10T00:00:00Z \
  --all

# Step 2: Inspect a specific ad's creative
agent-ads meta creatives preview --ad 120210987654321

# Step 3: Check if the pixel is healthy
agent-ads meta pixel-health get --pixel 9876543210

# Step 4: Check dataset match quality
agent-ads meta datasets get --id 5555555555
```

## 6. CI/Automation Pattern

Use in scripts or CI jobs with explicit error handling:

```bash
#!/bin/bash
set -euo pipefail

# Verify setup
agent-ads meta doctor --api -q

# Pull report
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --date-preset yesterday \
  --format csv \
  --output /data/yesterday.csv

echo "Report saved to /data/yesterday.csv"
```

Exit codes make it safe in `set -e` scripts: 0 = success, 1 = transport/internal, 2 = config/argument, 3 = Meta API error.

## 7. Piping and Composing

Combine with standard Unix tools:

```bash
# Pretty-print to less
agent-ads meta businesses list --pretty | less

# Filter with jq
agent-ads meta campaigns list --account act_1234567890 --all | jq '.[] | select(.status == "ACTIVE")'

# Count active campaigns
agent-ads meta campaigns list --account act_1234567890 --all | jq '[.[] | select(.effective_status == "ACTIVE")] | length'

# JSONL for line-by-line processing
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,spend \
  --date-preset last_7d \
  --format jsonl | while IFS= read -r line; do
    echo "$line" | jq -r '.campaign_id + ": $" + .spend'
  done
```
