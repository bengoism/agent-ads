# TikTok Reports & Insights

## Synchronous Reporting

Use `agent-ads tiktok insights query` for on-demand reporting via the `/report/integrated/get/` endpoint.

### Basic usage

```bash
agent-ads tiktok insights query \
  --advertiser-id 1234567890 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions stat_time_day \
  --metrics spend,impressions,clicks,cpc \
  --start-date 2026-03-01 \
  --end-date 2026-03-15
```

### Required parameters

| Flag | Description |
|------|-------------|
| `--advertiser-id` | Advertiser ID (or set default) |
| `--report-type` | `BASIC`, `AUDIENCE`, `PLAYABLE_MATERIAL`, `CATALOG` |
| `--dimensions` | Comma-separated dimension columns |
| `--metrics` | Comma-separated metric columns |

### Optional parameters

| Flag | Description |
|------|-------------|
| `--data-level` | `AUCTION_AD`, `AUCTION_ADGROUP`, `AUCTION_CAMPAIGN`, `AUCTION_ADVERTISER` |
| `--start-date` | Start date (YYYY-MM-DD) |
| `--end-date` | End date (YYYY-MM-DD) |
| `--filter` | JSON filter object |
| `--filter-file` | JSON file with filter |
| `--order-field` | Sort by this metric |
| `--order-type` | `ASC` or `DESC` |
| `--query-lifetime` | Query lifetime metrics |
| `--page` / `--page-size` | Pagination |
| `--all` / `--max-items` | Auto-paginate |

### Common dimension values

- `stat_time_day` — daily breakdown
- `stat_time_hour` — hourly breakdown
- `campaign_id` — by campaign
- `adgroup_id` — by ad group
- `ad_id` — by ad
- `country_code` — by country
- `gender` — by gender
- `age` — by age bucket

### Common metric values

- `spend`, `impressions`, `clicks`, `cpc`, `cpm`, `ctr`
- `conversion`, `cost_per_conversion`, `conversion_rate`
- `reach`, `frequency`
- `video_play_actions`, `video_watched_2s`, `video_watched_6s`

### Examples

```bash
# Daily campaign spend for the last week
agent-ads tiktok insights query \
  --advertiser-id 123 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions stat_time_day,campaign_id \
  --metrics spend,impressions,clicks \
  --start-date 2026-03-11 --end-date 2026-03-18

# Lifetime metrics
agent-ads tiktok insights query \
  --advertiser-id 123 \
  --report-type BASIC \
  --data-level AUCTION_AD \
  --dimensions ad_id \
  --metrics spend,impressions,conversion \
  --query-lifetime

# Export to CSV
agent-ads tiktok insights query \
  --advertiser-id 123 \
  --report-type BASIC \
  --data-level AUCTION_CAMPAIGN \
  --dimensions stat_time_day \
  --metrics spend,impressions \
  --start-date 2026-03-01 --end-date 2026-03-15 \
  --format csv --output report.csv
```

## Async Report Tasks

For large reports, use the async task API.

### Create a task

```bash
agent-ads tiktok report-runs submit \
  --advertiser-id 123 \
  --report-type BASIC \
  --data-level AUCTION_AD \
  --dimensions stat_time_day,ad_id \
  --metrics spend,impressions,clicks \
  --start-date 2026-01-01 --end-date 2026-03-01
```

Returns a task ID and a download URL when complete.

### Check status

```bash
agent-ads tiktok report-runs status \
  --advertiser-id 123 \
  --task-id TASK_ID_HERE
```

### Cancel a task

```bash
agent-ads tiktok report-runs cancel \
  --advertiser-id 123 \
  --task-id TASK_ID_HERE
```

The async report returns a download URL in the response data. Use `curl` or `wget` to fetch the file.
