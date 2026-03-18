# Command Topics

This file is the generated exhaustive CLI reference.
It is not the primary agent entrypoint.

Agents should start with `skills/agent-ads/SKILL.md`.
Humans should usually start with `README.md` or `agent-ads --help`.

Regenerate it with `npm run docs:generate`.

## Root Help

Canonical syntax uses space-separated subcommands. Colon-delimited forms are intentionally undocumented and unsupported.

```bash
agent-ads --help
```

```text
Unix-first multi-provider ads CLI

Usage: agent-ads [OPTIONS] <COMMAND>

Commands:
  providers  Inspect available and planned ad providers
  meta       Meta (Facebook/Instagram) Marketing API commands
  google     Google Ads provider namespace (not implemented yet)
  tiktok     TikTok Ads provider namespace (not implemented yet)
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
  -V, --version                            Print version
```

## Providers List

Inspect the currently available and planned provider namespaces.

```bash
agent-ads providers list
```

```json
[{"implemented":true,"provider":"meta","status":"available","summary":"Read-only Meta Marketing API support."},{"implemented":false,"provider":"google","status":"planned","summary":"Google Ads namespace is reserved but not implemented yet."},{"implemented":false,"provider":"tiktok","status":"planned","summary":"TikTok Ads namespace is reserved but not implemented yet."}]
```

## Meta Topic

The Meta provider owns all currently implemented ad commands.

```bash
agent-ads meta --help
```

```text
Meta (Facebook/Instagram) Marketing API commands

Usage: agent-ads meta [OPTIONS] <COMMAND>

Commands:
  businesses          List businesses accessible to your token
  ad-accounts         List ad accounts under a business
  campaigns           List campaigns in an ad account
  adsets              List ad sets in an ad account
  ads                 List ads in an ad account
  insights            Query performance insights (sync and async)
  report-runs         Manage async report run lifecycle
  creatives           Inspect ad creatives and previews
  activities          List account activity and change history
  custom-conversions  List custom conversion rules
  pixels              List tracking pixels
  datasets            Get dataset quality metrics
  pixel-health        Combined pixel health diagnostics
  auth                Manage stored auth token
  doctor              Verify auth, config, and API connectivity
  config              Inspect and validate configuration
  help                Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Businesses

```bash
agent-ads meta businesses --help
```

```text
List businesses accessible to your token

Usage: agent-ads meta businesses [OPTIONS] <COMMAND>

Commands:
  list  List businesses accessible to your token [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Ad Accounts

```bash
agent-ads meta ad-accounts --help
```

```text
List ad accounts under a business

Usage: agent-ads meta ad-accounts [OPTIONS] <COMMAND>

Commands:
  list  List ad accounts by scope [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Campaigns

```bash
agent-ads meta campaigns --help
```

```text
List campaigns in an ad account

Usage: agent-ads meta campaigns [OPTIONS] <COMMAND>

Commands:
  list  List objects in an ad account [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Insights

```bash
agent-ads meta insights --help
```

```text
Query performance insights (sync and async)

Usage: agent-ads meta insights [OPTIONS] <COMMAND>

Commands:
  query   Run a synchronous insights query
  export  Query insights with optional async mode
  help    Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Report Runs

```bash
agent-ads meta report-runs --help
```

```text
Manage async report run lifecycle

Usage: agent-ads meta report-runs [OPTIONS] <COMMAND>

Commands:
  submit   Submit an async report run
  status   Check async report run status
  results  Fetch completed report run results
  wait     Poll until a report run completes
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Creatives

```bash
agent-ads meta creatives --help
```

```text
Inspect ad creatives and previews

Usage: agent-ads meta creatives [OPTIONS] <COMMAND>

Commands:
  get      Fetch a creative by ID [aliases: cat]
  preview  Get rendered ad preview
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Activities

```bash
agent-ads meta activities --help
```

```text
List account activity and change history

Usage: agent-ads meta activities [OPTIONS] <COMMAND>

Commands:
  list  List account activity and change history [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Tracking

Tracking and measurement-health commands stay provider-specific; there is no shared cross-provider analytics abstraction.

```bash
agent-ads meta pixel-health --help
```

```text
Combined pixel health diagnostics

Usage: agent-ads meta pixel-health [OPTIONS] <COMMAND>

Commands:
  get   Get combined pixel health diagnostics [aliases: cat]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Auth

```bash
agent-ads meta auth --help
```

```text
Manage stored auth token

Usage: agent-ads meta auth [OPTIONS] <COMMAND>

Commands:
  set     Store the Meta token in the OS credential store
  status  Show auth source and secure storage status
  delete  Delete the stored Meta token
  help    Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Meta Config

```bash
agent-ads meta config --help
```

```text
Inspect and validate configuration

Usage: agent-ads meta config [OPTIONS] <COMMAND>

Commands:
  path      Show resolved config file path
  show      Show full resolved configuration
  validate  Validate config file
  help      Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. v25.0)
      --timeout-seconds <TIMEOUT_SECONDS>  HTTP request timeout in seconds
      --format <FORMAT>                    Output format [possible values: json, jsonl, csv]
      --output <OUTPUT>                    Write output to file (- for stdout)
      --pretty                             Pretty-print JSON output
      --envelope                           Include response metadata, paging, and warnings
      --include-meta                       Add metadata columns to CSV output
  -q, --quiet                              Suppress warnings and non-data output
  -v, --verbose...                         Increase log verbosity (-v info, -vv debug)
  -h, --help                               Print help
```

## Google Placeholder

Google is an explicit namespace, but it is not implemented yet.

```bash
agent-ads google
```

```json
{"implemented":false,"message":"google commands are not implemented yet. Use `agent-ads providers list` to inspect available providers or `agent-ads meta ...` for the current implementation.","provider":"google"}
```

## TikTok Placeholder

TikTok is an explicit namespace, but it is not implemented yet.

```bash
agent-ads tiktok
```

```json
{"implemented":false,"message":"tiktok commands are not implemented yet. Use `agent-ads providers list` to inspect available providers or `agent-ads meta ...` for the current implementation.","provider":"tiktok"}
```
