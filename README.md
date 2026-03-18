# agent-ads

Unix-first multi-provider ads CLI for analysts, agents, and CI jobs.

Query ad accounts, pull performance reports, inspect creatives, and diagnose tracking health — all from the terminal. Built in Rust, distributed through npm with prebuilt native binaries.

**Currently supported:** Meta (Facebook/Instagram) Marketing API (read-only).
**Namespaces reserved:** Google Ads, TikTok Ads (not yet implemented).

## Install

Global:

```bash
npm install -g agent-ads
```

Pinned in a project:

```bash
npm install agent-ads
```

From source:

```bash
git clone https://github.com/bengoism/agent-ads
cd agent-ads
npm install && npm run build:ts
cargo build
```

The npm package is a thin CLI launcher. There is no supported JavaScript API.

## Quick Start

### 1. Authenticate

Meta requires an access token. Set it as an environment variable or in a `.env` file:

```bash
export META_ADS_ACCESS_TOKEN=EAABs...
```

Or create a `.env` file in the current directory with the same variables.

### 2. Verify your setup

```bash
agent-ads meta doctor
```

```json
{
  "ok": true,
  "checks": [
    { "name": "env_file", "ok": true, "detail": "loaded auto-discovered env file from /work/.env" },
    { "name": "config_file", "ok": true, "detail": "using /work/agent-ads.config.json" },
    { "name": "access_token", "ok": true, "detail": "META_ADS_ACCESS_TOKEN is set" }
  ]
}
```

Add `--api` to also ping the Meta API and confirm your token works.

### 3. Discover your accounts

```bash
# List businesses you have access to
agent-ads meta businesses list

# List ad accounts under a business
agent-ads meta ad-accounts list --business-id 1234567890
```

```json
[
  { "id": "act_1234567890", "name": "My Ad Account", "account_status": 1, "currency": "USD" }
]
```

### 4. Explore campaigns

```bash
agent-ads meta campaigns list --account act_1234567890
```

### 5. Pull a performance report

```bash
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --since 2026-03-01 \
  --until 2026-03-16 \
  --time-increment 1
```

```json
[
  {
    "campaign_id": "120210123456",
    "campaign_name": "Spring Sale",
    "impressions": "4520",
    "clicks": "312",
    "spend": "48.75",
    "date_start": "2026-03-01",
    "date_stop": "2026-03-01"
  }
]
```

### 6. Export to CSV

```bash
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --date-preset last_7d \
  --format csv \
  --output report.csv
```

## Command Overview

Providers are always explicit: `agent-ads <provider> <command>`. There is no cross-provider abstraction.

### Top-level

| Command | Description |
|---------|-------------|
| `agent-ads providers list` | Show available and planned providers |
| `agent-ads meta ...` | Meta Marketing API commands |
| `agent-ads google` | Placeholder (not implemented) |
| `agent-ads tiktok` | Placeholder (not implemented) |

### Meta: Discovery

| Command | Description |
|---------|-------------|
| `meta businesses list` | List businesses accessible to your token |
| `meta ad-accounts list` | List ad accounts under a business |
| `meta campaigns list` | List campaigns in an ad account |
| `meta adsets list` | List ad sets in an ad account |
| `meta ads list` | List ads in an ad account |

### Meta: Reporting

| Command | Description |
|---------|-------------|
| `meta insights query` | Synchronous insights query |
| `meta insights export` | Insights with optional async mode (`--async --wait`) |
| `meta report-runs submit` | Submit an async report job |
| `meta report-runs status` | Check async report status |
| `meta report-runs wait` | Poll until async report completes |
| `meta report-runs results` | Fetch completed report results |

### Meta: Creative & Changes

| Command | Description |
|---------|-------------|
| `meta creatives get` | Fetch a creative by ID |
| `meta creatives preview` | Get rendered ad preview (by creative or ad ID) |
| `meta activities list` | List account activity/change history |

### Meta: Tracking & Measurement

| Command | Description |
|---------|-------------|
| `meta custom-conversions list` | List custom conversions |
| `meta pixels list` | List pixels |
| `meta datasets get` | Get dataset quality metrics |
| `meta pixel-health get` | Combined pixel diagnostics (metadata + stats) |

### Meta: Config & Diagnostics

| Command | Description |
|---------|-------------|
| `meta config path` | Show resolved config file path |
| `meta config show` | Show full resolved configuration |
| `meta config validate` | Validate config file |
| `meta doctor` | Verify auth, config, and (optionally) API connectivity |

## Global Flags

These flags work with any command:

| Flag | Description |
|------|-------------|
| `--config <path>` | Config file path (default: `agent-ads.config.json`) |
| `--env-file <path>` | Env file path (default: `./.env`) |
| `--api-base-url <url>` | Override Meta API base URL |
| `--api-version <version>` | Override API version (e.g. `v25.0`) |
| `--timeout-seconds <n>` | HTTP request timeout |
| `--format json\|jsonl\|csv` | Output format (default: `json`) |
| `--output <path>` | Write output to file instead of stdout (`-` for stdout) |
| `--pretty` | Pretty-print JSON output |
| `--envelope` | Include response metadata, paging info, and warnings |
| `--include-meta` | Add metadata columns to CSV output |
| `-q, --quiet` | Suppress warnings and non-data output |
| `-v, --verbose` | Increase log verbosity (repeat for debug: `-vv`) |

## Pagination

List commands support cursor-based pagination:

| Flag | Description |
|------|-------------|
| `--page-size <n>` | Number of items per API request |
| `--cursor <token>` | Resume from a specific cursor |
| `--all` | Automatically follow all pages |
| `--max-items <n>` | Stop after collecting N total items |

Use `--all` to fetch everything, or `--max-items 100` to cap results. Without either flag, you get one page of results and can use `--envelope` to see the paging cursor for the next page.

## Configuration

### Precedence

CLI flags > shell environment > `.env` file > `agent-ads.config.json`

### Config file

Copy the example to get started:

```bash
cp agent-ads.config.json.example agent-ads.config.json
```

Supported keys:

```json
{
  "output_format": "json",
  "providers": {
    "meta": {
      "api_base_url": "https://graph.facebook.com",
      "api_version": "v25.0",
      "timeout_seconds": 60,
      "default_business_id": "1234567890",
      "default_account_id": "act_1234567890"
    }
  }
}
```

Setting `default_business_id` and `default_account_id` lets you omit `--business-id` and `--account` from commands.

### Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `META_ADS_ACCESS_TOKEN` | Yes | Meta API access token |

Secrets are never read from config files or CLI flags — only from environment variables.

## Output

Default output is **data-only JSON** on stdout. Errors are JSON on stderr.

### Formats

```bash
# Default: JSON array
agent-ads meta businesses list
# [{"id":"123","name":"My Business"}]

# One JSON object per line (good for streaming/piping)
agent-ads meta businesses list --format jsonl
# {"id":"123","name":"My Business"}

# CSV (good for spreadsheets)
agent-ads meta businesses list --format csv
# id,name
# 123,My Business

# Pretty-printed JSON
agent-ads meta businesses list --pretty
```

### Envelope mode

Add `--envelope` to wrap data with response metadata, paging cursors, and warnings:

```bash
agent-ads meta businesses list --envelope --pretty
```

```json
{
  "data": [{ "id": "123", "name": "My Business" }],
  "meta": { "api_version": "v25.0", "endpoint": "/me/businesses" },
  "paging": { "cursors": { "before": "...", "after": "..." }, "next": "..." }
}
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Transport or internal error |
| 2 | Config or argument error |
| 3 | Meta API error |

## Docs Map

| Audience | Start here |
|----------|------------|
| Humans | This README, then `agent-ads --help` |
| AI agents | [skills/agent-ads/SKILL.md](skills/agent-ads/SKILL.md) |
| Meta provider deep-dive | [skills/agent-ads/references/meta.md](skills/agent-ads/references/meta.md) |
| Full CLI reference (generated) | [docs/command-topics.md](docs/command-topics.md) |
| Live help | `agent-ads --help`, `agent-ads meta --help`, `agent-ads meta insights query --help` |

## Skills

If you use this repo with an agent runtime (Claude Code, Codex, etc.), install the public skill:

```bash
npx skills add bengoism/agent-ads
```

The repo-local source of truth for the skill is [skills/agent-ads/SKILL.md](skills/agent-ads/SKILL.md).

## Development

```bash
npm install
npm run build:ts
cargo fmt
cargo test
npm run test:smoke
npm run docs:generate

# Run locally from source
cargo run -p agent_ads_cli -- --help
cargo run -p agent_ads_cli -- meta doctor
```

## Publishing

The release workflow publishes six npm packages:

- `agent-ads` — CLI launcher
- `agent-ads-darwin-arm64` / `agent-ads-darwin-x64` — macOS binaries
- `agent-ads-linux-arm64-gnu` / `agent-ads-linux-x64-gnu` — Linux binaries
- `agent-ads-windows-x64-msvc` — Windows binary

For a brand-new release, bootstrap with an npm publish token first. After the first successful publish, switch the packages to [npm trusted publishing](https://docs.npmjs.com/generating-provenance-statements):

```bash
npm run release:trust
```

## License

MIT
