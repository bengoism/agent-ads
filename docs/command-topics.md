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
  providers  
  meta       
  google     Google Ads provider namespace. Commands are not implemented yet.
  tiktok     TikTok Ads provider namespace. Commands are not implemented yet.
  help       Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>                    
      --env-file <ENV_FILE>                
      --api-base-url <API_BASE_URL>        
      --api-version <API_VERSION>          
      --timeout-seconds <TIMEOUT_SECONDS>  
      --format <FORMAT>                    [possible values: json, jsonl, csv]
      --output <OUTPUT>                    
      --pretty                             
      --envelope                           
      --include-meta                       
  -q, --quiet                              
  -v, --verbose...                         
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
Usage: agent-ads meta <COMMAND>

Commands:
  businesses          
  ad-accounts         
  campaigns           
  adsets              
  ads                 
  insights            
  report-runs         
  creatives           
  activities          
  custom-conversions  
  pixels              
  datasets            
  pixel-health        
  doctor              
  config              
  help                Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Businesses

```bash
agent-ads meta businesses --help
```

```text
Usage: agent-ads meta businesses <COMMAND>

Commands:
  list  [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Ad Accounts

```bash
agent-ads meta ad-accounts --help
```

```text
Usage: agent-ads meta ad-accounts <COMMAND>

Commands:
  list  [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Campaigns

```bash
agent-ads meta campaigns --help
```

```text
Usage: agent-ads meta campaigns <COMMAND>

Commands:
  list  [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Insights

```bash
agent-ads meta insights --help
```

```text
Usage: agent-ads meta insights <COMMAND>

Commands:
  query   
  export  
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Report Runs

```bash
agent-ads meta report-runs --help
```

```text
Usage: agent-ads meta report-runs <COMMAND>

Commands:
  submit   
  status   
  results  
  wait     
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Creatives

```bash
agent-ads meta creatives --help
```

```text
Usage: agent-ads meta creatives <COMMAND>

Commands:
  get      [aliases: cat]
  preview  
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Activities

```bash
agent-ads meta activities --help
```

```text
Usage: agent-ads meta activities <COMMAND>

Commands:
  list  [aliases: ls]
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Tracking

Tracking and measurement-health commands stay provider-specific; there is no shared cross-provider analytics abstraction.

```bash
agent-ads meta pixel-health --help
```

```text
Usage: agent-ads meta pixel-health <COMMAND>

Commands:
  get   [aliases: cat]
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Meta Config

```bash
agent-ads meta config --help
```

```text
Usage: agent-ads meta config <COMMAND>

Commands:
  path      
  show      
  validate  
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
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
