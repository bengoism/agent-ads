# CLAUDE.md

Project conventions for agents working in this repo.

## What this is

`agent-ads` is a Rust CLI distributed via npm. It queries read-only ad platform APIs for Meta, Google Ads, TikTok, Pinterest, LinkedIn, and X. The architecture is provider-explicit: every command starts with `agent-ads <provider> ...`.

## Repo structure

- `crates/agent_ads_core/` — Rust library: HTTP client, auth, config, output, all endpoint logic
- `crates/agent_ads_cli/` — Rust binary: clap CLI that produces `agent-ads`
- `npm/agent-ads/` — npm package: thin JS launcher that spawns the correct native binary
- `npm/platform/` — per-platform npm packages carrying prebuilt binaries
- `skills/agent-ads/` — public skill (SKILL.md + reference docs for agents)
- `docs/command-topics.md` — auto-generated CLI reference (do not edit manually)
- `dist/` — local build artifacts (gitignored on CI)

## Build commands

```bash
npm install                    # install JS deps
npm run build:ts               # compile TS launcher
cargo build                    # build Rust binary
cargo test                     # run Rust tests
cargo fmt                      # format Rust code
npm run test:smoke             # smoke test the built CLI
npm run docs:generate          # regenerate docs/command-topics.md from CLI help output
```

## Key conventions

- **Provider-explicit commands**: always `agent-ads meta ...`, never `agent-ads campaigns ...`
- **No colon syntax**: `agent-ads meta insights query`, not `agent-ads meta:insights:query`
- **Secrets from secure storage**: `META_ADS_ACCESS_TOKEN` / `GOOGLE_ADS_*` / `TIKTOK_ADS_ACCESS_TOKEN` come from shell env or the OS credential store, never from flags or config files
- **Secret precedence**: shell env > OS credential store
- **Non-secret precedence**: CLI flags > shell env > `agent-ads.config.json`
- **Data-only output**: stdout is raw JSON by default (no envelope). Use `--envelope` for metadata/paging wrapper
- **Errors on stderr**: always JSON format
- **Exit codes**: 0=success, 1=transport/internal, 2=config/argument, 3=Meta API, 4=TikTok API, 5=Google API, 6=Pinterest API, 7=LinkedIn API, 8=X API
- **Read-only**: the CLI does not create, update, or delete any ad objects
- **No agent attribution or promotion**: never add Claude, Claude Code, Anthropic, or any agent as an author/co-author/reviewer in commits or PRs, and do not include self-promotional references to Claude Code or Anthropic unless the user explicitly asks for them

## Adding new commands

1. Add the endpoint and client updates in the relevant provider file under `crates/agent_ads_core/src/`
2. Add clap args and dispatch in the matching provider file under `crates/agent_ads_cli/src/`
3. Always include an `about = "..."` on the clap subcommand
4. Run `cargo fmt && cargo test`
5. Update the relevant reference doc in `skills/agent-ads/references/`
6. Run `npm run docs:generate` to update the generated CLI reference

## Testing

- `cargo test` runs unit tests (clap arg parsing, help rendering)
- `npm run test:smoke` runs the built binary with `--help` and `providers list`
- There are no integration tests that hit real provider APIs (would require real credentials)

## Docs structure

- `README.md` — human-facing project overview
- `CLAUDE.md` — this file, for agents
- `skills/agent-ads/SKILL.md` — agent skill entrypoint
- `skills/agent-ads/references/meta.md` — Meta routing guide
- `skills/agent-ads/references/meta-*.md` — Meta topic-specific reference files
- `skills/agent-ads/references/google.md` — Google routing guide
- `skills/agent-ads/references/google-workflows.md` — Google end-to-end recipes
- `skills/agent-ads/references/tiktok.md` — TikTok routing guide
- `skills/agent-ads/references/tiktok-*.md` — TikTok topic-specific reference files
- `skills/agent-ads/references/pinterest.md` — Pinterest routing guide
- `skills/agent-ads/references/linkedin.md` — LinkedIn routing guide
- `skills/agent-ads/references/x.md` — X routing guide
- `docs/command-topics.md` — generated exhaustive CLI reference

Do not edit `docs/command-topics.md` by hand. Edit clap `about`/`help` strings in Rust source and regenerate.
