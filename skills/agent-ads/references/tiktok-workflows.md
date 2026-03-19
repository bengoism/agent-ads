# TikTok Workflows

End-to-end recipes for TikTok Ads. Each workflow is a sequence of commands you can run in order.

## 1. Account Discovery

Start from your token and discover all advertisers:

```bash
# Step 1: List your advertisers (requires app credentials)
agent-ads tiktok advertisers list \
  --app-id $TIKTOK_ADS_APP_ID \
  --app-secret $TIKTOK_ADS_APP_SECRET

# Step 2: Get details for a specific advertiser
agent-ads tiktok advertisers info --advertiser-id 1234567890

# Step 3: List campaigns
agent-ads tiktok campaigns list --advertiser-id 1234567890
```

## 2. Daily Performance Report

Pull daily campaign performance for a date range, exported as CSV:

```bash
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions stat_time_day,campaign_id \
  --metrics spend,impressions,clicks,cpc,ctr \
  --start-date 2026-03-01 \
  --end-date 2026-03-16 \
  --format csv \
  --output campaign-daily.csv
```

## 3. Ad-Level Report With Breakdowns

Detailed ad-level performance with demographic breakdowns:

```bash
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type AUDIENCE \
  --data-level AUCTION_AD \
  --dimensions ad_id,gender,age \
  --metrics spend,impressions,clicks,conversion \
  --start-date 2026-03-01 \
  --end-date 2026-03-16 \
  --all
```

## 4. Large Async Export

For reports too large for sync queries:

```bash
# Step 1: Submit the async task
agent-ads tiktok report-runs submit \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_AD \
  --dimensions stat_time_day,ad_id \
  --metrics spend,impressions,clicks \
  --start-date 2026-01-01 \
  --end-date 2026-03-01

# Step 2: Check status (note the task_id from step 1)
agent-ads tiktok report-runs status \
  --advertiser-id 1234567890 \
  --task-id TASK_ID_HERE

# Step 3: When complete, the status response contains a download URL.
# Use curl or wget to fetch the file.
```

## 5. Creative and Tracking Audit

Inspect creative assets and tracking setup:

```bash
# Step 1: List video creatives
agent-ads tiktok creatives videos --advertiser-id 1234567890 --all

# Step 2: Check pixels
agent-ads tiktok pixels list --advertiser-id 1234567890

# Step 3: List custom audiences
agent-ads tiktok audiences list --advertiser-id 1234567890 --all
```

## 6. CI / Automation Pattern

```bash
#!/bin/bash
set -euo pipefail

# Refresh token (TikTok tokens expire every 24 hours)
agent-ads tiktok auth refresh \
  --app-id "$TIKTOK_ADS_APP_ID" \
  --app-secret "$TIKTOK_ADS_APP_SECRET"

# Verify setup
agent-ads tiktok doctor --api -q

# Pull report
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions stat_time_day,campaign_id \
  --metrics spend,impressions,clicks \
  --start-date "$(date -d yesterday +%Y-%m-%d)" \
  --end-date "$(date +%Y-%m-%d)" \
  --format csv \
  --output /data/tiktok-yesterday.csv

echo "Report saved to /data/tiktok-yesterday.csv"
```

Exit codes make it safe in `set -e` scripts: 0 = success, 1 = transport/internal, 2 = config/argument, 4 = TikTok API error.

## 7. Piping and Composing

```bash
# Pretty-print to less
agent-ads tiktok campaigns list --advertiser-id 1234567890 --pretty | less

# Filter with jq
agent-ads tiktok campaigns list --advertiser-id 1234567890 --all \
  | jq '.[] | select(.primary_status == "STATUS_ENABLE")'

# JSONL for line-by-line processing
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions campaign_id \
  --metrics spend,impressions \
  --start-date 2026-03-01 --end-date 2026-03-16 \
  --format jsonl | while IFS= read -r line; do
    echo "$line" | jq -r '.dimensions.campaign_id + ": $" + .metrics.spend'
  done
```
