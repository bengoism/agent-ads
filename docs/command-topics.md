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
  auth       Inspect auth status and route into provider setup
  meta       Meta (Facebook/Instagram) Marketing API commands
  google     Google Ads commands
  tiktok     TikTok Business API commands
  pinterest  Pinterest Ads API commands
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
[{"implemented":true,"provider":"meta","status":"available","summary":"Read-only Meta Marketing API support."},{"implemented":true,"provider":"google","status":"available","summary":"Read-only Google Ads support with native GAQL."},{"implemented":true,"provider":"tiktok","status":"available","summary":"Read-only TikTok Business API support."},{"implemented":true,"provider":"pinterest","status":"available","summary":"Read-only Pinterest Ads API support."}]
```

## Root Auth

Inspect aggregated auth status or launch guided local setup.

```bash
agent-ads auth --help
```

```text
Inspect auth status and route into provider setup

Usage: agent-ads auth [OPTIONS] [COMMAND]

Commands:
  status  Show aggregated auth status across implemented providers
  help    Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## TikTok Topic

The TikTok provider covers the TikTok Business API.

```bash
agent-ads tiktok --help
```

```text
TikTok Business API commands

Usage: agent-ads tiktok [OPTIONS] <COMMAND>

Commands:
  advertisers  List and inspect advertiser accounts
  campaigns    List campaigns
  adgroups     List ad groups
  ads          List ads
  insights     Query performance insights (synchronous)
  report-runs  Manage async report tasks
  creatives    Search video and image creative assets
  pixels       List tracking pixels
  audiences    List custom audiences
  auth         Manage stored auth credentials
  doctor       Verify auth, config, and API connectivity
  config       Inspect and validate configuration
  help         Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## TikTok Advertisers

```bash
agent-ads tiktok advertisers --help
```

```text
List and inspect advertiser accounts

Usage: agent-ads tiktok advertisers [OPTIONS] <COMMAND>

Commands:
  list  List authorized advertisers [aliases: ls]
  info  Get advertiser account details [aliases: cat]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## TikTok Campaigns

```bash
agent-ads tiktok campaigns --help
```

```text
List campaigns

Usage: agent-ads tiktok campaigns [OPTIONS] <COMMAND>

Commands:
  list  List objects for an advertiser [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## TikTok Insights

```bash
agent-ads tiktok insights --help
```

```text
Query performance insights (synchronous)

Usage: agent-ads tiktok insights [OPTIONS] <COMMAND>

Commands:
  query  Run a synchronous insights query
  help   Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## TikTok Report Runs

```bash
agent-ads tiktok report-runs --help
```

```text
Manage async report tasks

Usage: agent-ads tiktok report-runs [OPTIONS] <COMMAND>

Commands:
  submit  Create an async report task
  status  Check async report task status
  cancel  Cancel an async report task
  help    Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## TikTok Creatives

```bash
agent-ads tiktok creatives --help
```

```text
Search video and image creative assets

Usage: agent-ads tiktok creatives [OPTIONS] <COMMAND>

Commands:
  videos  Search video assets
  images  Get image info by IDs
  help    Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## TikTok Auth

```bash
agent-ads tiktok auth --help
```

```text
Manage stored auth credentials

Usage: agent-ads tiktok auth [OPTIONS] <COMMAND>

Commands:
  set      Store TikTok auth credentials in the OS credential store
  status   Show auth source and secure storage status
  delete   Delete the stored TikTok credentials
  refresh  Refresh the access token using a stored refresh token
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## TikTok Config

```bash
agent-ads tiktok config --help
```

```text
Inspect and validate configuration

Usage: agent-ads tiktok config [OPTIONS] <COMMAND>

Commands:
  path      Show resolved config file path
  show      Show full resolved configuration
  validate  Validate config file
  help      Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Pinterest Topic

The Pinterest provider covers the Pinterest Ads API.

```bash
agent-ads pinterest --help
```

```text
Pinterest Ads API commands

Usage: agent-ads pinterest [OPTIONS] <COMMAND>

Commands:
  ad-accounts          List and inspect Pinterest ad accounts
  campaigns            List campaigns
  adgroups             List ad groups
  ads                  List ads
  analytics            Query Pinterest analytics synchronously
  report-runs          Manage async Pinterest report runs
  audiences            List and inspect audiences
  targeting-analytics  Query targeting analytics
  auth                 Manage stored Pinterest auth credentials
  doctor               Verify auth, config, and API connectivity
  config               Inspect and validate configuration
  help                 Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Pinterest Ad Accounts

```bash
agent-ads pinterest ad-accounts --help
```

```text
List and inspect Pinterest ad accounts

Usage: agent-ads pinterest ad-accounts [OPTIONS] <COMMAND>

Commands:
  list  List ad accounts [aliases: ls]
  get   Get a single ad account [aliases: cat]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Pinterest Campaigns

```bash
agent-ads pinterest campaigns --help
```

```text
List campaigns

Usage: agent-ads pinterest campaigns [OPTIONS] <COMMAND>

Commands:
  list  List campaigns for an ad account [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Pinterest Analytics

```bash
agent-ads pinterest analytics --help
```

```text
Query Pinterest analytics synchronously

Usage: agent-ads pinterest analytics [OPTIONS] <COMMAND>

Commands:
  query  Run a synchronous analytics query
  help   Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Pinterest Report Runs

```bash
agent-ads pinterest report-runs --help
```

```text
Manage async Pinterest report runs

Usage: agent-ads pinterest report-runs [OPTIONS] <COMMAND>

Commands:
  submit  Submit an async report request
  status  Check async report status
  wait    Poll until the report reaches a terminal state
  help    Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Pinterest Audiences

```bash
agent-ads pinterest audiences --help
```

```text
List and inspect audiences

Usage: agent-ads pinterest audiences [OPTIONS] <COMMAND>

Commands:
  list  List audiences for an ad account [aliases: ls]
  get   Get a single audience [aliases: cat]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Pinterest Auth

```bash
agent-ads pinterest auth --help
```

```text
Manage stored Pinterest auth credentials

Usage: agent-ads pinterest auth [OPTIONS] <COMMAND>

Commands:
  set      Store Pinterest app credentials and tokens in the OS credential store
  status   Show auth source and secure storage status
  delete   Delete stored Pinterest credentials
  refresh  Refresh the access token using the stored refresh token
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Pinterest Config

```bash
agent-ads pinterest config --help
```

```text
Inspect and validate configuration

Usage: agent-ads pinterest config [OPTIONS] <COMMAND>

Commands:
  path      Show resolved config file path
  show      Show full resolved configuration
  validate  Validate config file
  help      Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Google Topic

The Google provider covers read-only Google Ads and native GAQL access.

```bash
agent-ads google --help
```

```text
Google Ads commands

Usage: agent-ads google [OPTIONS] <COMMAND>

Commands:
  customers  List accessible customers and customer hierarchies
  campaigns  List campaigns
  adgroups   List ad groups
  ads        List ads
  gaql       Run Google Ads Query Language (GAQL) requests
  auth       Manage stored Google Ads credentials
  doctor     Verify auth, config, and API connectivity
  config     Inspect and validate configuration
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Google Customers

```bash
agent-ads google customers --help
```

```text
List accessible customers and customer hierarchies

Usage: agent-ads google customers [OPTIONS] <COMMAND>

Commands:
  list       List customers accessible to your Google Ads credentials [aliases: ls]
  hierarchy  List a customer hierarchy via customer_client GAQL
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Google Campaigns

```bash
agent-ads google campaigns --help
```

```text
List campaigns

Usage: agent-ads google campaigns [OPTIONS] <COMMAND>

Commands:
  list  List campaigns for a customer [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Google GAQL

```bash
agent-ads google gaql --help
```

```text
Run Google Ads Query Language (GAQL) requests

Usage: agent-ads google gaql [OPTIONS] <COMMAND>

Commands:
  search         Run a paged GAQL search request
  search-stream  Run a streamed GAQL search request
  help           Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Google Auth

```bash
agent-ads google auth --help
```

```text
Manage stored Google Ads credentials

Usage: agent-ads google auth [OPTIONS] <COMMAND>

Commands:
  set     Store Google Ads credentials in the OS credential store
  status  Show auth source and secure storage status
  delete  Delete stored Google Ads credentials
  help    Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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

## Google Config

```bash
agent-ads google config --help
```

```text
Inspect and validate configuration

Usage: agent-ads google config [OPTIONS] <COMMAND>

Commands:
  path      Show resolved config file path
  show      Show full resolved configuration
  validate  Validate config file
  help      Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    Config file path [default: agent-ads.config.json]
      --api-base-url <API_BASE_URL>        Override API base URL
      --api-version <API_VERSION>          Override API version (e.g. Meta v25.0 or Google v23)
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
