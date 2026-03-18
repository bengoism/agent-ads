# TikTok Accounts & Objects

## Advertisers

### List authorized advertisers

Requires app credentials (the OAuth endpoint needs them):

```bash
agent-ads tiktok advertisers list \
  --app-id YOUR_APP_ID \
  --app-secret YOUR_APP_SECRET

# Or with env vars
export TIKTOK_ADS_APP_ID=...
export TIKTOK_ADS_APP_SECRET=...
agent-ads tiktok advertisers list
```

### Get advertiser details

```bash
agent-ads tiktok advertisers info \
  --advertiser-id 1234567890,9876543210

# With custom fields
agent-ads tiktok advertisers info \
  --advertiser-id 1234567890 \
  --fields display_name,company,status
```

## Campaigns

```bash
# List campaigns for an advertiser
agent-ads tiktok campaigns list --advertiser-id 1234567890

# With fields and pagination
agent-ads tiktok campaigns list \
  --advertiser-id 1234567890 \
  --fields campaign_id,campaign_name,budget,status \
  --page-size 50

# Auto-paginate all
agent-ads tiktok campaigns list \
  --advertiser-id 1234567890 \
  --all

# With filtering
agent-ads tiktok campaigns list \
  --advertiser-id 1234567890 \
  --filter '{"primary_status":"STATUS_ENABLE"}'
```

## Ad Groups

```bash
# List ad groups
agent-ads tiktok adgroups list --advertiser-id 1234567890

# With filtering by campaign
agent-ads tiktok adgroups list \
  --advertiser-id 1234567890 \
  --filter '{"campaign_ids":["123456"]}'
```

## Ads

```bash
# List ads
agent-ads tiktok ads list --advertiser-id 1234567890

# With fields
agent-ads tiktok ads list \
  --advertiser-id 1234567890 \
  --fields ad_id,ad_name,adgroup_id,status
```

## Common Patterns

### Default advertiser ID

Set `TIKTOK_ADS_DEFAULT_ADVERTISER_ID` or add `default_advertiser_id` to `providers.tiktok` in your config file to avoid repeating `--advertiser-id` on every command.

### Filtering

TikTok filtering is a JSON object passed via `--filter`:

```bash
# By status
--filter '{"primary_status":"STATUS_ENABLE"}'

# By IDs
--filter '{"campaign_ids":["123","456"]}'

# From a file
--filter-file filters.json
```

### Pagination

TikTok uses page-number pagination:

```bash
# Page 2, 50 items per page
--page 2 --page-size 50

# Auto-paginate everything
--all

# Stop after 100 items
--all --max-items 100
```
