# Meta Guide

Use this file when the user is operating the implemented Meta provider inside `agent-ads`.

## Start Here

| Need | First command | Read |
|------|---------------|------|
| Check local setup or output behavior | `agent-ads meta config show` or `agent-ads meta doctor` | [meta-auth-and-output.md](meta-auth-and-output.md) |
| Discover businesses or ad accounts | `agent-ads meta businesses list` | [meta-accounts-and-objects.md](meta-accounts-and-objects.md) |
| List campaigns, ad sets, or ads | `agent-ads meta campaigns list --account <account-id>` | [meta-accounts-and-objects.md](meta-accounts-and-objects.md) |
| Run a sync insights query | `agent-ads meta insights query ...` | [meta-reports.md](meta-reports.md) |
| Run or inspect an async job | `agent-ads meta insights export ...` or `agent-ads meta report-runs ...` | [meta-reports.md](meta-reports.md) |
| Inspect creatives or diagnose changes | `agent-ads meta creatives preview ...` or `agent-ads meta activities list ...` | [meta-creative-and-changes.md](meta-creative-and-changes.md) |
| Check pixels, datasets, or measurement health | `agent-ads meta pixel-health get ...` | [meta-tracking.md](meta-tracking.md) |
| Follow a full recipe instead of a command-family reference | none | [meta-workflows.md](meta-workflows.md) |

## Operating Rules

- The CLI is non-interactive. Supply all required flags.
- Secrets only come from environment variables:
  - `META_ADS_ACCESS_TOKEN` is required
  - `META_ADS_APP_SECRET` is optional
- `agent-ads` auto-loads `./.env` if present, and `--env-file <path>` overrides the location.
- Prefer `--format json` for automation, `jsonl` for row streaming, and `csv` for exports.
- Output is data-only by default. Add `--envelope` when you need request metadata, paging, or warnings.
- Meta config precedence is `CLI flags > shell env > .env > agent-ads.config.json`.
- Error payloads are JSON on stderr.
- Exit codes:
  - `0` success
  - `1` transport or internal failure
  - `2` config or argument failure
  - `3` Meta API failure

## Common Mistakes

- Forgetting `META_ADS_ACCESS_TOKEN` before running API commands
- Passing both `--account` and `--object` to `insights` commands
- Using `--action-breakdowns` without `actions` in `--fields`
- Treating default JSON output as if it always includes `meta`, `paging`, or `warnings`
- Assuming `pixel-health` is a first-party EMQ passthrough rather than a combined diagnostic view
- Forgetting `--all` or `--max-items` when paginating list and report results
