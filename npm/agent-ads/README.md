# agent-ads

CLI for querying ad platform APIs from the terminal. Meta, Google Ads, and TikTok supported. Read-only. Built in Rust, shipped as prebuilt native binaries.

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

Google and TikTok follow the same pattern: `agent-ads google ...`, `agent-ads tiktok ...`.

## Documentation

Full docs, all three providers, examples, and configuration: [GitHub repo](https://github.com/bengoism/agent-ads)

## License

MIT
