# Meta Auth And Output

Use this file for local setup, authentication, config resolution, and output behavior.

## Authentication

Two environment variables control Meta API access:

| Variable | Required | Purpose |
|----------|----------|---------|
| `META_ADS_ACCESS_TOKEN` | Yes | Bearer token for all Meta API calls |

### Required permissions

| Permission | Needed for |
|------------|------------|
| `ads_read` | All `--account` commands: campaigns, insights, creatives, pixels |
| `business_management` | `businesses list` and business-scoped `ad-accounts list` discovery |

Both are read-only — no write access is granted. `ad-accounts list` without a business ID can still fall back to `/me/adaccounts`, but business-scoped discovery (`--business-id`, `--scope owned`, `--scope pending-client`) still needs `business_management`. Generate a token at the [Graph API Explorer](https://developers.facebook.com/tools/explorer/) with the permissions above.

Secrets are **never** read from CLI flags or config files. Persistent secrets live in the OS credential store, and shell env remains available for overrides or CI.

### Setting up auth

Option 1 — store the token in the OS credential store:

```bash
agent-ads meta auth set
```

Option 2 — shell environment override:

```bash
export META_ADS_ACCESS_TOKEN=EAABs...
```

On Linux, persistent secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet. In headless environments, use the shell variable for that process.

### Verifying auth

```bash
# Check config and env without hitting the API
agent-ads meta doctor

# Also ping the API to confirm the token works
agent-ads meta doctor --api
```

Example `doctor` output:

```json
{
  "ok": true,
  "checks": [
    { "name": "credential_store", "ok": true, "detail": "stored Meta token found in OS credential store" },
    { "name": "config_file", "ok": true, "detail": "using /work/agent-ads.config.json" },
    { "name": "access_token", "ok": true, "detail": "using stored Meta token from the OS credential store" },
    { "name": "api_ping", "ok": true, "detail": "token accepted by Meta API; sampled 1 business record(s)" }
  ]
}
```

## Config Resolution

Precedence (highest to lowest):

1. Shell `META_ADS_ACCESS_TOKEN`
2. OS credential store for the token

For non-secret config:

1. CLI flags (`--api-version v24.0`)
2. Shell environment variables
3. `agent-ads.config.json` file values

### Config file

Default path: `agent-ads.config.json` in the current directory. Override with `--config <path>`.

Supported keys under `providers.meta`:

| Key | Default | Description |
|-----|---------|-------------|
| `api_base_url` | `https://graph.facebook.com` | Meta Graph API base URL |
| `api_version` | `v25.0` | API version |
| `timeout_seconds` | `60` | HTTP request timeout |
| `default_business_id` | none | Fallback for `--business-id` |
| `default_account_id` | none | Fallback for `--account` |
| `output_format` | `json` | Default output format |

### Inspecting config

```bash
# Show the resolved config file path
agent-ads meta config path

# Show the full resolved configuration (all sources merged)
agent-ads meta config show

# Validate the config file parses correctly
agent-ads meta config validate
```

## Output

### Formats

| Flag | Behavior | Best for |
|------|----------|----------|
| `--format json` (default) | JSON array or object | Automation, piping to `jq` |
| `--format jsonl` | One JSON object per line | Streaming, line-by-line processing |
| `--format csv` | CSV with header row | Spreadsheets, data import |

### Default output (data-only)

By default, stdout contains only the data — no metadata wrapper:

```json
[
  { "id": "act_123", "name": "Agency Account" }
]
```

### Envelope mode

Add `--envelope` to wrap data with response metadata, paging cursors, and warnings:

```json
{
  "data": [{ "id": "act_123", "name": "Agency Account" }],
  "meta": {
    "api_version": "v25.0",
    "endpoint": "/me/adaccounts",
    "object_id": "me"
  },
  "paging": { "cursors": { "before": "...", "after": "..." }, "next": "..." }
}
```

Use `--envelope` when you need to extract the paging cursor for manual pagination, or when you want to see request metadata and warnings.

### Other output flags

| Flag | What it does |
|------|-------------|
| `--pretty` | Pretty-print JSON output |
| `--include-meta` | Add `api_version`, `endpoint`, `object_id` as columns in CSV output |
| `--output <path>` | Write to a file instead of stdout (`-` for explicit stdout) |
| `-q, --quiet` | Suppress warnings on stderr |

### Error output

Errors are always JSON on stderr:

```json
{ "error": { "kind": "api", "message": "Invalid OAuth 2.0 Access Token", "code": 190 } }
```

## Exit Codes

| Code | Meaning | Example |
|------|---------|---------|
| `0` | Success | Normal response |
| `1` | Transport or internal failure | Network timeout, serialization error |
| `2` | Config or argument failure | Missing token, invalid flags |
| `3` | Meta API failure | Token expired, rate limit, invalid field |
