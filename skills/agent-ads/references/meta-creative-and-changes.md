# Meta Creative And Changes

Use this file for inspecting ad creative content and reviewing account activity history.

## Creative Lookups

### Get creative by ID

Fetch the full creative object (story spec, asset feed, thumbnail):

```bash
agent-ads meta creatives get --id 120210123456789
```

Default fields: `id`, `name`, `object_story_spec`, `asset_feed_spec`, `thumbnail_url`.

```bash
# With custom fields
agent-ads meta creatives get --id 120210123456789 --fields id,name,body,image_url,call_to_action_type
```

### Preview creative

Get the rendered ad preview. You can target by creative ID or ad ID:

```bash
# By creative ID
agent-ads meta creatives preview --creative 120210123456789

# By ad ID (resolves the ad's creative automatically)
agent-ads meta creatives preview --ad 120210987654321
```

Default fields: `body` (the rendered preview HTML/payload).

Optional flags:

| Flag | Description |
|------|-------------|
| `--ad-format <format>` | Ad format to preview (e.g. `DESKTOP_FEED_STANDARD`, `MOBILE_FEED_STANDARD`) |
| `--render-type <type>` | Render type (e.g. `FALLBACK`) |
| `--fields <list>` | Custom fields |

When using `--ad`, the CLI first resolves the ad to its creative ID, then calls the preview edge. A warning is emitted noting this resolution step.

## Activities (Change History)

List account activity logs to answer "what changed and when?"

```bash
agent-ads meta activities list --account act_1234567890
```

Default fields: `id`, `event_time`, `event_type`, `category`, `object_type`, `translated_event_type`.

### Filtering activities

| Flag | Description |
|------|-------------|
| `--since <timestamp>` | Start time (ISO 8601, e.g. `2026-03-10T00:00:00Z`) |
| `--until <timestamp>` | End time |
| `--category <category>` | Filter by category (e.g. `AD`, `CAMPAIGN`, `BUDGET`) |
| `--data-source <source>` | Filter by data source |
| `--oid <object-id>` | Filter by specific object ID |
| `--business-id <id>` | Filter by business |
| `--add-children` | Include child object activities |

### Pagination

Activities support standard pagination: `--page-size`, `--cursor`, `--all`, `--max-items`.

Forensic investigations often need more than one page. Always use `--all` or `--max-items` explicitly.

### Example: find all changes in the last 24 hours

```bash
agent-ads meta activities list \
  --account act_1234567890 \
  --since 2026-03-17T00:00:00Z \
  --all \
  --fields id,event_time,event_type,category,object_type,translated_event_type,extra_data
```

## When to Use What

| Question | Command |
|----------|---------|
| "What does this ad look like?" | `creatives preview --ad <id>` |
| "What's the raw creative spec?" | `creatives get --id <id>` |
| "What changed in this account recently?" | `activities list --account <id> --since <time> --all` |
| "Who changed this specific campaign?" | `activities list --account <id> --oid <campaign-id> --all` |

Note: `activities list` does **not** support `--date-preset` or `--time-range-file`. Use `--since` and `--until` directly.
