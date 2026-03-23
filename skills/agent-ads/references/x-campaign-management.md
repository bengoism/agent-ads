# X Campaign Management

Use this guide when the task is account discovery, campaigns, line items, or other account-scoped management surfaces in X Ads.

## Account Discovery

```bash
agent-ads x accounts list
agent-ads x accounts get --account-id 18ce54d4x5t
agent-ads x authenticated-user-access get --account-id 18ce54d4x5t
```

`authenticated-user-access get` is the fastest way to confirm the current credential bundle can see one account and what role it has there.

## Campaign Hierarchy

```bash
agent-ads x campaigns list --account-id 18ce54d4x5t
agent-ads x campaigns get --account-id 18ce54d4x5t --campaign-id c123

agent-ads x line-items list --account-id 18ce54d4x5t
agent-ads x line-items get --account-id 18ce54d4x5t --line-item-id l123
```

These stay close to the X Ads API resource model. Do not rename them into Meta or TikTok concepts.

## Funding And Delivery Surfaces

```bash
agent-ads x funding-instruments list --account-id 18ce54d4x5t
agent-ads x promotable-users list --account-id 18ce54d4x5t
agent-ads x promoted-accounts list --account-id 18ce54d4x5t
agent-ads x targeting-criteria list --account-id 18ce54d4x5t
```

- `funding-instruments` inspects billing sources attached to the account.
- `promotable-users` shows identities that can be promoted from the account.
- `promoted-accounts` inspects follows/account-promotion surfaces.
- `targeting-criteria` exposes provider-native targeting objects tied to line items.

## Account Scope

Most X commands are account-scoped. If you omit `--account-id`, the CLI falls back to:

1. explicit CLI flag
2. `X_ADS_DEFAULT_ACCOUNT_ID`
3. `providers.x.default_account_id`

If none are set, the command fails fast with a config error instead of guessing.
