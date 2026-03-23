---
name: agent-ads
description: >
  Read-only CLI for Meta, Google, TikTok, Pinterest, LinkedIn, and X ad APIs. JSON to stdout.
  Use when the user wants explicit provider-prefixed commands for auth, account discovery,
  reporting, creatives, tracking, audiences, or troubleshooting.
---

# Agent Ads

`agent-ads` is a read-only CLI for querying ad platform APIs. Every command outputs JSON to stdout. Keep commands provider-specific: `agent-ads <provider> <command>`.

## Common Tasks

Use the skill for direct, provider-specific CLI work across the supported ad platforms.

- Check available providers: `agent-ads providers list`
- Check auth status across providers: `agent-ads auth status`
- Discover Meta businesses and ad accounts
- List Google Ads customers or run GAQL queries
- Query TikTok, Pinterest, LinkedIn, or X reporting surfaces
- Inspect creatives, pixels, audiences, and other provider-native objects
- Run `agent-ads <provider> doctor` before troubleshooting auth or config

## Command Syntax Rules

Every command starts with `agent-ads <provider>`. The CLI stays provider-first because each platform has its own auth model, object graph, and reporting semantics.

- Canonical: `agent-ads meta insights query ...`
- Never: `agent-ads insights query ...`
- Never: `agent-ads meta:insights:query`

## Provider Routing

Start with the provider guide that matches the platform you need.

| Provider | First command | Reference guide |
|----------|---------------|-----------------|
| `meta` | `agent-ads meta --help` | [references/meta.md](references/meta.md) |
| `google` | `agent-ads google --help` | [references/google.md](references/google.md) |
| `tiktok` | `agent-ads tiktok --help` | [references/tiktok.md](references/tiktok.md) |
| `pinterest` | `agent-ads pinterest --help` | [references/pinterest.md](references/pinterest.md) |
| `linkedin` | `agent-ads linkedin --help` | [references/linkedin.md](references/linkedin.md) |
| `x` | `agent-ads x --help` | [references/x.md](references/x.md) |

Check live: `agent-ads providers list`

For a cross-provider auth summary or guided local setup, use `agent-ads auth`.

Load only the provider guide you need. Do not preload all reference files.

## Shared Behavior

These rules apply to every provider.

### Output

- **stdout**: data-only JSON by default
- **stderr**: errors as JSON, warnings as plain text
- **`--envelope`**: wraps stdout with `{ "data": ..., "meta": ..., "paging": ... }`
- **Common flags**: `--format json|jsonl|csv`, `--output <path>`, `--pretty`

### Config precedence

- Secrets: shell env > OS credential store. Never from flags or config files.
- Non-secrets: CLI flags > shell env > `agent-ads.config.json`
- Guided local auth setup: `agent-ads auth`
- Persistent provider auth: `agent-ads <provider> auth set`
- Shell override / CI: provider-specific env vars
- Linux secure storage requires a running Secret Service provider such as GNOME Keyring or KWallet

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Transport or internal error |
| 2 | Config or argument error |
| 3 | Meta API error |
| 4 | TikTok API error |
| 5 | Google API error |
| 6 | Pinterest API error |
| 7 | LinkedIn API error |
| 8 | X API error |

### Global flags

| Flag | Default | What it does |
|------|---------|--------------|
| `--config <path>` | `agent-ads.config.json` | Config file path |
| `--format json\|jsonl\|csv` | `json` | Output format |
| `--output <path>` | stdout | Write to file (`-` for stdout) |
| `--pretty` | off | Pretty-print JSON |
| `--envelope` | off | Wrap data with metadata, paging, warnings |
| `--include-meta` | off | Add metadata columns in CSV mode |
| `--api-version <v>` | provider default | Provider API version override |
| `--timeout-seconds <n>` | `60` | HTTP request timeout |
| `-q` / `-v` | warn | Quiet mode or verbose logging (`-vv` for debug) |

Pagination flags differ by provider. Use `agent-ads <provider> --help` or the provider guide to confirm the exact shape.

## Common Issues

These are the first checks worth making when commands fail.

| Problem | What's happening | Fix |
|---------|------------------|-----|
| "My token expired" | Meta tokens can be short-lived; TikTok access tokens expire every 24 hours; Pinterest uses OAuth refresh tokens | Meta: regenerate at [Graph API Explorer](https://developers.facebook.com/tools/explorer/) then run `meta auth set`. TikTok: run `tiktok auth refresh`. Pinterest: run `pinterest auth refresh` |
| "I don't know my account ID" | Most providers require an explicit account, customer, or advertiser scope | Discover it first: Meta `businesses list` / `ad-accounts list`; Google `customers list`; TikTok `advertisers list`; Pinterest `ad-accounts list`; LinkedIn `ad-accounts list`; X `accounts list` |
| "Permission denied" | The token is missing required scopes or account access | Re-check the provider auth guide and reissue credentials with the required read scope |
| "doctor says credential store unavailable" | No OS keychain is available on this machine | Use provider-specific shell env vars for that session or CI job |
| "<provider> says an ID is required" | The command is scoped and no default is configured | Pass the provider-specific ID flag or set the matching default in `providers.<provider>` config |
| "Command not found" | The CLI is missing or not on `PATH` | Run `agent-ads --version`; if needed, install with `npm install -g agent-ads` |

## Stop Conditions

Keep the command model explicit and provider-native.

- Do not drop the provider prefix (`agent-ads meta ...`, not `agent-ads ...`)
- Do not invent cross-provider abstractions or shared schemas
- Do not reuse one provider's auth env vars for another provider
- Do not guess flag names; confirm with `agent-ads <provider> <command> --help`
