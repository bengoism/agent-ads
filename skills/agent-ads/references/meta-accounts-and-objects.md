# Meta Accounts And Objects

Use this file for business discovery, ad-account listing, and object metadata sync.

## Businesses

```bash
agent-ads meta businesses list
agent-ads meta businesses list --all --fields id,name,verification_status
```

## Ad Accounts

Scopes:

- `accessible`
- `owned`
- `pending-client`

```bash
agent-ads meta ad-accounts list --business-id <business-id> --scope accessible
```

## Campaigns, Ad Sets, Ads

All object list commands accept:

- `--account <id>`
- `--fields <comma,separated,fields>` or `--fields-file <path>`
- `--page-size <n>`
- `--cursor <cursor>`
- `--all`
- `--max-items <n>`

```bash
agent-ads meta campaigns list --account <account-id> --all
agent-ads meta adsets list --account <account-id> --all
agent-ads meta ads list --account <account-id> --all
```

## Notes

- Account ids accept either `123` or `act_123`.
- Output normalizes account ids to `act_<id>`.
- Use `--max-items` when you want bounded pagination without handling cursors yourself.
