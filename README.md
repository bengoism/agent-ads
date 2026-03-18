# `agent-ads`

Unix-first multi-provider ads CLI for analysts, agents, and CI jobs.

Built in Rust. Distributed through npm with prebuilt native binaries.

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
npm install
npm run build:ts
cargo build
```

The npm package is a CLI launcher only. There is no supported JavaScript API.

## Current Status

- `meta`: implemented
- `google`: namespace reserved, not implemented
- `tiktok`: namespace reserved, not implemented

## Mental Model

- Providers are always explicit: `agent-ads meta ...`, `agent-ads google ...`, `agent-ads tiktok ...`
- There is no provider-agnostic ads abstraction layer in the CLI
- Shared behavior is limited to output, packaging, and top-level dispatch
- Canonical command syntax is space-separated. Colon-delimited forms are intentionally unsupported.

## Quick Start

Inspect the available providers:

```bash
agent-ads providers list
```

Authenticate for Meta with env vars:

```bash
export META_ADS_ACCESS_TOKEN=...
export META_ADS_APP_SECRET=... # optional, enables appsecret_proof
```

Or create a local `.env` in the current working directory:

```dotenv
META_ADS_ACCESS_TOKEN=...
META_ADS_APP_SECRET=...
```

Inspect local setup before making API calls:

```bash
agent-ads meta config show
agent-ads meta doctor
```

Use `--env-file <path>` if your env file is not `./.env`.

Run one discovery command and one report command:

```bash
agent-ads meta businesses list

agent-ads meta insights query \
  --account 1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --since 2026-03-01 \
  --until 2026-03-16 \
  --time-increment 1
```

Use [agent-ads.config.json.example](agent-ads.config.json.example) as the starting point for local defaults.

## Docs Map

- Humans should start with this README.
- Agents should start with [skills/agent-ads/SKILL.md](skills/agent-ads/SKILL.md).
- The Meta provider guide lives in [skills/agent-ads/references/meta.md](skills/agent-ads/references/meta.md).
- The generated exhaustive CLI reference is [docs/command-topics.md](docs/command-topics.md).
- Live command help is always available through `agent-ads --help` and `agent-ads meta --help`.

## Config And Output

- Default config file: `agent-ads.config.json`
- Default env file: `./.env`
- Meta config precedence: `CLI flags > shell environment > .env > agent-ads.config.json`
- Required env var for Meta API calls: `META_ADS_ACCESS_TOKEN`
- Existing shell env vars are never overwritten by `.env`
- Default stdout is data-only JSON
- Add `--envelope` when you need response metadata, paging, or warnings
- Errors are JSON on stderr

## Skills

If you use this repo with Codex or another agent runtime, install the public skill with:

```bash
npx skills add bengoism/agent-ads
```

The repo-local source of truth for that skill is [skills/agent-ads/SKILL.md](skills/agent-ads/SKILL.md).

## Publishing

The release workflow publishes six npm packages:

- `agent-ads`
- `agent-ads-darwin-arm64`
- `agent-ads-darwin-x64`
- `agent-ads-linux-arm64-gnu`
- `agent-ads-linux-x64-gnu`
- `agent-ads-win32-x64-msvc`

For a brand-new release, bootstrap with an npm publish token first:

1. Create an npm automation or publish token for the `bengoism` account.
2. Add it to GitHub Actions as the `NPM_PUBLISH_TOKEN` secret in `bengoism/agent-ads`.
3. Push a `v*` tag to run the release workflow.

After the first successful publish, switch the packages to npm trusted publishing:

```bash
npm install -g npm@^11.10.0
npm login
npm run release:trust
```

That configures all six packages to trust `.github/workflows/release.yml` from `bengoism/agent-ads`.
After that, future releases can run without `NPM_PUBLISH_TOKEN`.

## Development

```bash
npm install
npm run build:ts
npm run docs:generate
cargo fmt
cargo test
npm run test:smoke
cargo run -p meta_ads_cli -- --help
cargo run -p meta_ads_cli -- meta --help
```
