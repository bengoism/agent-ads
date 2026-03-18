---
name: agent-ads
description: >
  Operate the provider-first ads CLI from the terminal. Use when the user wants
  to inspect available ad providers, route work into the implemented Meta
  provider, or keep ad-provider commands explicit inside `agent-ads`.
---

# Agent Ads

Use this as the only public skill entrypoint.

## Contract

- Canonical syntax is `agent-ads <provider> ...`
- Providers are explicit: `agent-ads meta ...`, `agent-ads google ...`, `agent-ads tiktok ...`
- Do not invent provider-agnostic commands like `agent-ads campaigns list`
- Do not use colon-delimited command forms like `agent-ads meta:insights:query`
- Shared flags and output behavior live at the umbrella level
- Provider auth, config, object IDs, and semantics stay provider-owned

## Provider Status

- `meta`: implemented
- `google`: namespace reserved, not implemented
- `tiktok`: namespace reserved, not implemented

## Start Here

| Need | First command | Read |
|------|---------------|------|
| Confirm which providers exist | `agent-ads providers list` | this file |
| Work with Meta accounts, reports, creatives, or tracking | `agent-ads meta --help` | [references/meta.md](references/meta.md) |
| Work with Google or TikTok | none | explain that the namespace exists but is not implemented yet |
| Plan future provider support | none | this file |

## Meta Routing

- If the provider is `meta`, read [references/meta.md](references/meta.md) first.
- Then load only the specific Meta reference file linked from that guide.
- Keep Meta auth and object semantics inside the Meta provider. Do not reuse them for future providers.

## Future Providers

- If the user is planning future provider support, keep provider concepts separate.
- Do not normalize provider concepts into shared campaign, report, or measurement schemas.

## Stop Conditions

- Do not drop the provider prefix when calling commands.
- Do not normalize provider concepts into a shared campaign/report schema.
- Do not reuse Meta auth env vars for future providers.
