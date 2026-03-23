# agent-ads

CLI for querying ad platform APIs from the terminal. Meta, Google Ads, TikTok, and Pinterest supported. Read-only. Built in Rust, shipped as prebuilt native binaries.

## Install

```bash
npm install -g agent-ads
```

## Quick Start

```bash
# Store your Meta access token once
agent-ads meta auth set

# Verify your setup
agent-ads meta doctor

# Pull a performance report
agent-ads meta insights query \
  --account act_1234567890 \
  --level campaign \
  --fields campaign_id,campaign_name,impressions,clicks,spend \
  --date-preset last_7d
```

Google, TikTok, and Pinterest follow the same pattern: `agent-ads google ...`, `agent-ads tiktok ...`, `agent-ads pinterest ...`.

## Claude Code Skill

agent-ads ships as a skill for Claude Code. Install the CLI, then add the skill:

```bash
npx skills add https://github.com/bengoism/agent-ads --skill agent-ads
```

Your agent can now query any supported ad platform using plain English.

## Documentation

Full docs, all providers, examples, and configuration: [agent-ads.dev](https://agent-ads.dev/)

## License

MIT
