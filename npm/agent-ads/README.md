# agent-ads

CLI for querying ad platform APIs from the terminal. Meta, Google Ads, TikTok, Pinterest, and LinkedIn supported. Read-only. Built in Rust, shipped as prebuilt native binaries.

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

Every provider follows the same shape: `agent-ads <provider> <command>`.

| Provider | Authenticate | Start with |
| --- | --- | --- |
| Meta | `agent-ads meta auth set` | `agent-ads meta insights query --account act_1234567890 --level campaign --fields campaign_id,spend --date-preset last_7d` |
| Google Ads | `agent-ads google auth set` | `agent-ads google gaql search --customer-id 123-456-7890 --query "SELECT campaign.name FROM campaign"` |
| TikTok | `agent-ads tiktok auth set --full` | `agent-ads tiktok insights query --advertiser-id 1234567890 --data-level AUCTION_CAMPAIGN --metrics spend,clicks` |
| Pinterest | `agent-ads pinterest auth set` | `agent-ads pinterest analytics query --ad-account-id 1234567890 --columns IMPRESSION_1,SPEND_IN_DOLLAR --start-date 2026-03-01 --end-date 2026-03-16` |
| LinkedIn | `agent-ads linkedin auth set` | `agent-ads linkedin analytics query --finder statistics --account-id 1234567890 --pivot CAMPAIGN --fields impressions,clicks,costInLocalCurrency` |

Verify any configured provider with `agent-ads <provider> doctor`.

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
