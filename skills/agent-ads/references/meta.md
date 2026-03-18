# Meta Guide

This is the routing guide for the Meta provider. Read this first when the user wants to work with Meta (Facebook/Instagram) ads. Then load only the specific reference file linked below.

## Route to the Right Reference

| Task | First command | Reference file |
|------|---------------|----------------|
| Set up auth, check config, understand output | `agent-ads meta doctor` | [meta-auth-and-output.md](meta-auth-and-output.md) |
| Discover businesses and ad accounts | `agent-ads meta businesses list` | [meta-accounts-and-objects.md](meta-accounts-and-objects.md) |
| List campaigns, ad sets, or ads | `agent-ads meta campaigns list --account <id>` | [meta-accounts-and-objects.md](meta-accounts-and-objects.md) |
| Run a performance report (sync) | `agent-ads meta insights query ...` | [meta-reports.md](meta-reports.md) |
| Run a large async report | `agent-ads meta insights export --async --wait ...` | [meta-reports.md](meta-reports.md) |
| Manage async report lifecycle | `agent-ads meta report-runs submit/wait/results` | [meta-reports.md](meta-reports.md) |
| Inspect ad creatives | `agent-ads meta creatives preview --ad <id>` | [meta-creative-and-changes.md](meta-creative-and-changes.md) |
| Review account change history | `agent-ads meta activities list --account <id>` | [meta-creative-and-changes.md](meta-creative-and-changes.md) |
| Check pixels, datasets, or measurement health | `agent-ads meta pixel-health get --pixel <id>` | [meta-tracking.md](meta-tracking.md) |
| Follow an end-to-end recipe | — | [meta-workflows.md](meta-workflows.md) |

Load only the reference file you need. Do not preload all of them.

## Quick Reference

### Auth (secure storage or shell env — never from flags or config files)

| Variable | Required | Purpose |
|----------|----------|---------|
| `META_ADS_ACCESS_TOKEN` | No | Shell override / CI fallback bearer token |

Persistent local auth is stored with `agent-ads meta auth set`.

### Config precedence

Token precedence: shell env > OS credential store

Non-secret precedence: CLI flags > shell env > `agent-ads.config.json`

### Output defaults

- stdout: data-only JSON (no wrapper)
- stderr: errors as JSON, warnings as text
- `--envelope` adds metadata/paging wrapper
- `--format json|jsonl|csv`

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Transport or internal failure |
| 2 | Config or argument failure |
| 3 | Meta API failure |

## Common Mistakes

- Forgetting `META_ADS_ACCESS_TOKEN` before running API commands
- Passing both `--account` and `--object` to insights commands (they are mutually exclusive)
- Using `--action-breakdowns` without `actions` in `--fields`
- Treating default JSON output as if it includes `meta`, `paging`, or `warnings` (it doesn't — use `--envelope`)
- Assuming `pixel-health` is a raw EMQ passthrough (it's a combined diagnostic view)
- Forgetting `--all` or `--max-items` when paginating (you only get one page by default)
