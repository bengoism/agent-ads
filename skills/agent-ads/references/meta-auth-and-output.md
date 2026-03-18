# Meta Auth And Output

Use this file for local setup, auth, config precedence, and output behavior.

## Authentication

- Required env var: `META_ADS_ACCESS_TOKEN`
- Optional env var: `META_ADS_APP_SECRET`
- Secrets are not read from flags or config files.
- `agent-ads` auto-loads `./.env` from the current working directory if it exists.
- Use `--env-file <path>` to load a different env file explicitly.
- Existing shell env vars win over values from `.env`.

## Config Resolution

- CLI flags win over shell environment values.
- Shell environment values win over `.env`.
- `.env` values win over `agent-ads.config.json`.
- Default config file name: `agent-ads.config.json`
- Default env file name: `.env` in the current working directory

Supported config keys:

- `api_base_url`
- `api_version`
- `timeout_seconds`
- `default_business_id`
- `default_account_id`
- `output_format`

Inspect config without hitting Meta:

```bash
agent-ads meta config path
agent-ads meta config show
agent-ads meta config validate
agent-ads meta doctor
```

## Output

- `--format json` is the default and safest for automation.
- `--format jsonl` emits one data row per line for array responses.
- `--format csv` emits row data only by default.
- `--envelope` restores the response wrapper with `meta`, `paging`, and `warnings`.
- `--include-meta` adds metadata columns to CSV output.

Default JSON output:

```json
[
  {
    "id": "act_123",
    "name": "Agency Account"
  }
]
```

Envelope mode:

```json
{
  "data": [
    {
      "id": "act_123",
      "name": "Agency Account"
    }
  ],
  "meta": {
    "api_version": "v25.0",
    "endpoint": "/1234567890/ad_accounts",
    "object_id": "1234567890"
  },
  "paging": {
    "next": "..."
  }
}
```

Error output is JSON on stderr.

## Exit Codes

- `0` success
- `1` transport or internal failure
- `2` config or argument failure
- `3` Meta API failure
