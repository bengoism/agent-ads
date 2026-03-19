# Google Workflows

End-to-end recipes for Google Ads. Each workflow is a sequence of commands you can run in order.

## 1. Account Discovery

Start from your credentials and discover all accessible customers:

```bash
# Step 1: List all accessible customers
agent-ads google customers list --all --pretty

# Step 2: Explore a customer hierarchy (manager accounts)
agent-ads google customers hierarchy --customer-id 1234567890 --all
```

If you query through a manager account, set `login_customer_id` in config or pass `GOOGLE_ADS_LOGIN_CUSTOMER_ID`.

## 2. Campaign Overview

```bash
# List campaigns
agent-ads google campaigns list --customer-id 1234567890

# List ad groups
agent-ads google adgroups list --customer-id 1234567890

# List ads
agent-ads google ads list --customer-id 1234567890
```

## 3. GAQL Performance Report

Pull campaign metrics with a native GAQL query:

```bash
agent-ads google gaql search \
  --customer-id 1234567890 \
  --query "SELECT campaign.id, campaign.name, metrics.impressions, metrics.clicks, metrics.cost_micros FROM campaign WHERE segments.date DURING LAST_7_DAYS"
```

For a longer query, put it in a file:

```bash
agent-ads google gaql search \
  --customer-id 1234567890 \
  --query-file campaign-report.sql \
  --all
```

## 4. Streaming Large Results to CSV

`search-stream` avoids page-token overhead for large result sets:

```bash
agent-ads google gaql search-stream \
  --customer-id 1234567890 \
  --query "SELECT ad_group.id, ad_group.name, metrics.impressions, metrics.clicks FROM ad_group WHERE segments.date DURING LAST_30_DAYS" \
  --format csv \
  --output adgroup-report.csv
```

## 5. CI / Automation Pattern

```bash
#!/bin/bash
set -euo pipefail

# Verify setup
agent-ads google doctor --api -q

# Pull yesterday's campaign data
agent-ads google gaql search-stream \
  --customer-id 1234567890 \
  --query "SELECT campaign.id, campaign.name, metrics.impressions, metrics.clicks, metrics.cost_micros FROM campaign WHERE segments.date = '$(date -d yesterday +%Y-%m-%d)'" \
  --format csv \
  --output /data/google-yesterday.csv

echo "Report saved to /data/google-yesterday.csv"
```

Exit codes make it safe in `set -e` scripts: 0 = success, 1 = transport/internal, 2 = config/argument, 5 = Google API error.

## 6. Piping and Composing

```bash
# Pretty-print to less
agent-ads google campaigns list --customer-id 1234567890 --pretty | less

# Filter with jq
agent-ads google campaigns list --customer-id 1234567890 --all \
  | jq '.[] | select(.campaign.status == "ENABLED")'

# Count enabled campaigns
agent-ads google campaigns list --customer-id 1234567890 --all \
  | jq '[.[] | select(.campaign.status == "ENABLED")] | length'
```
