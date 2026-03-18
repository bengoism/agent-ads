# Meta Tracking

Use this file for custom conversions, pixels, datasets, and measurement-health diagnostics.

## Custom Conversions

List custom conversion rules for an ad account:

```bash
agent-ads meta custom-conversions list --account act_1234567890
agent-ads meta custom-conversions list --account act_1234567890 --all
```

Default fields: `id`, `name`, `custom_event_type`, `rule`, `creation_time`.

## Pixels

List pixels attached to an ad account:

```bash
agent-ads meta pixels list --account act_1234567890
agent-ads meta pixels list --account act_1234567890 --all --fields id,name,last_fired_time,match_rate_approx
```

Default fields: `id`, `name`, `owner_ad_account`, `last_fired_time`, `match_rate_approx`, `event_stats`.

## Datasets

Get quality metrics for a specific dataset (offline event set):

```bash
agent-ads meta datasets get --id 1234567890
agent-ads meta datasets get --id 1234567890 --fields id,name,event_stats,match_rate_approx,collection_rate
```

Default fields: `id`, `name`, `event_stats`, `last_fired_time`, `match_rate_approx`.

Note: `--id` is the dataset ID, not an ad account ID.

## Pixel Health (Combined Diagnostics)

`pixel-health get` is a combined diagnostic view — it fetches pixel metadata **and** the pixel stats edge in a single call, then returns them together:

```bash
agent-ads meta pixel-health get --pixel 1234567890
```

Returns:

```json
{
  "pixel": {
    "id": "1234567890",
    "name": "My Pixel",
    "match_rate_approx": 0.45,
    "event_stats": "...",
    "last_fired_time": "2026-03-17T12:00:00Z"
  },
  "stats": [...]
}
```

### Optional filters

| Flag | Description |
|------|-------------|
| `--aggregation <value>` | Aggregation level for stats |
| `--event <name>` | Filter to a specific event (e.g. `Purchase`, `Lead`) |
| `--event-source <source>` | Filter by event source |
| `--start-time <timestamp>` | Stats start time |
| `--end-time <timestamp>` | Stats end time |
| `--fields <list>` | Override default pixel fields |

### Example: check health for a specific event

```bash
agent-ads meta pixel-health get \
  --pixel 1234567890 \
  --event Purchase \
  --start-time 2026-03-01 \
  --end-time 2026-03-17
```

## When to Use What

| Question | Command |
|----------|---------|
| "What pixels does this account have?" | `pixels list --account <id>` |
| "What custom conversions are set up?" | `custom-conversions list --account <id>` |
| "Is the pixel firing correctly?" | `pixel-health get --pixel <id>` |
| "What's the dataset match rate?" | `datasets get --id <id>` |
| "What events has the pixel received recently?" | `pixel-health get --pixel <id> --event Purchase` |

Note: `pixel-health` is a practical diagnostics view built by the CLI. It is **not** a raw passthrough of Meta's Event Match Quality (EMQ) API. It combines the pixel node metadata with the `/stats` edge to give a unified health picture.
