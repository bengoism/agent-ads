# X Auth And Config

Use this guide when the task is authentication, env vars, config precedence, or health checks for X Ads.

## Auth Commands

```bash
agent-ads x auth set
agent-ads x auth status
agent-ads x auth delete
```

`agent-ads x auth set --stdin` reads four newline-delimited values from stdin in this order:

1. consumer key
2. consumer secret
3. access token
4. access token secret

## Shell Overrides

```bash
export X_ADS_CONSUMER_KEY=consumer-key
export X_ADS_CONSUMER_SECRET=consumer-secret
export X_ADS_ACCESS_TOKEN=access-token
export X_ADS_ACCESS_TOKEN_SECRET=access-token-secret
export X_ADS_DEFAULT_ACCOUNT_ID=18ce54d4x5t
export X_ADS_API_BASE_URL=https://ads-api.x.com
export X_ADS_API_VERSION=12
export X_ADS_TIMEOUT_SECONDS=60
export X_ADS_OUTPUT_FORMAT=json
```

Secrets come from shell env first, then the OS credential store. Never put secrets in `agent-ads.config.json`.

## Config File Shape

```json
{
  "providers": {
    "x": {
      "api_base_url": "https://ads-api.x.com",
      "api_version": "12",
      "timeout_seconds": 60,
      "default_account_id": "18ce54d4x5t",
      "output_format": "json"
    }
  }
}
```

## Health And Inspection

```bash
agent-ads x doctor
agent-ads x doctor --api
agent-ads x config path
agent-ads x config show
agent-ads x config validate
```

- `doctor` reports credential-store access, config file resolution, and whether each required secret is present.
- `doctor --api` adds a lightweight authenticated API request.
- `config show` returns resolved non-secret config plus auth source metadata.

## Root Auth

For guided local setup across providers, use the shared root commands:

```bash
agent-ads auth
agent-ads auth status
agent-ads auth clear
```

X appears there as the canonical provider name `x`. Do not use a `twitter` alias.
