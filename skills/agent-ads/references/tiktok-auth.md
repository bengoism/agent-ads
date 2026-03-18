# TikTok Auth, Config & Output

## Authentication

TikTok uses the custom `Access-Token` HTTP header (not Bearer, not query param).

### Token lifecycle

- **Access tokens** expire every **24 hours**
- **Refresh tokens** expire after **1 year**
- Use `agent-ads tiktok auth refresh` to rotate tokens automatically

### Setting up auth

```bash
# Option 1: Store in OS credential store (recommended)
agent-ads tiktok auth set
# Prompts for access token securely

# Option 2: Store both access and refresh tokens
agent-ads tiktok auth set --refresh-token
# Prompts for access token, then refresh token

# Option 3: Pipe from stdin
echo "$TOKEN" | agent-ads tiktok auth set --stdin

# Option 3b: Pipe both access + refresh tokens from stdin
printf '%s\n%s\n' "$ACCESS_TOKEN" "$REFRESH_TOKEN" | \
  agent-ads tiktok auth set --stdin --refresh-token

# Option 4: Shell env (CI / ephemeral)
export TIKTOK_ADS_ACCESS_TOKEN=your_token_here
```

### Token refresh

```bash
# Refresh using stored refresh token + app credentials
agent-ads tiktok auth refresh \
  --app-id YOUR_APP_ID \
  --app-secret YOUR_APP_SECRET

# Or use env vars
export TIKTOK_ADS_APP_ID=your_app_id
export TIKTOK_ADS_APP_SECRET=your_app_secret
export TIKTOK_ADS_REFRESH_TOKEN=your_refresh_token
agent-ads tiktok auth refresh
```

`auth refresh` resolves the refresh token from `TIKTOK_ADS_REFRESH_TOKEN` first, then falls back to the OS credential store.

This stores the new access token (and updated refresh token) in the OS credential store.

### Auth commands

| Command | What it does |
|---------|-------------|
| `agent-ads tiktok auth set` | Store access token |
| `agent-ads tiktok auth set --refresh-token` | Store both tokens |
| `agent-ads tiktok auth status` | Show token source and storage status |
| `agent-ads tiktok auth delete` | Delete all stored TikTok tokens |
| `agent-ads tiktok auth refresh --app-id ... --app-secret ...` | Rotate access token |

## Configuration

### Config file

Add a `providers.tiktok` section to `agent-ads.config.json`:

```json
{
  "providers": {
    "tiktok": {
      "api_base_url": "https://business-api.tiktok.com",
      "api_version": "v1.3",
      "timeout_seconds": 60,
      "default_advertiser_id": "1234567890",
      "output_format": "json"
    }
  }
}
```

### Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `TIKTOK_ADS_ACCESS_TOKEN` | — | Short-lived access token |
| `TIKTOK_ADS_REFRESH_TOKEN` | — | Long-lived refresh token |
| `TIKTOK_ADS_APP_ID` | — | App ID (for refresh and advertiser list) |
| `TIKTOK_ADS_APP_SECRET` | — | App secret (for refresh and advertiser list) |
| `TIKTOK_ADS_API_BASE_URL` | `https://business-api.tiktok.com` | Override API base URL |
| `TIKTOK_ADS_API_VERSION` | `v1.3` | Override API version |
| `TIKTOK_ADS_TIMEOUT_SECONDS` | `60` | HTTP timeout |
| `TIKTOK_ADS_DEFAULT_ADVERTISER_ID` | — | Default advertiser for all commands |
| `TIKTOK_ADS_OUTPUT_FORMAT` | `json` | Default output format |

### Config commands

| Command | What it does |
|---------|-------------|
| `agent-ads tiktok config path` | Show resolved config file path |
| `agent-ads tiktok config show` | Show full resolved configuration |
| `agent-ads tiktok config validate` | Validate config file |

### Doctor

```bash
# Basic check
agent-ads tiktok doctor

# With live API ping (requires default_advertiser_id)
agent-ads tiktok doctor --api
```

## TikTok API Response Format

All TikTok API responses have this envelope:

```json
{
  "code": 0,
  "message": "OK",
  "request_id": "...",
  "data": { ... }
}
```

- `code: 0` = success
- `code: 20001` = partial success
- `code: 40xxx` = client error (auth, params)
- `code: 50xxx` = server error
- `code: 61000` = rate limited

The CLI strips this envelope and returns only the `data` field by default. Use `--envelope` to see metadata.
