# X Creatives

Use this guide when the task is promoted tweets, media, cards, or other creative surfaces in X Ads.

## Promoted Tweets And Timelines

```bash
agent-ads x promoted-tweets list --account-id 18ce54d4x5t
agent-ads x promoted-tweets get --account-id 18ce54d4x5t --promoted-tweet-id pt123
agent-ads x scoped-timeline list --account-id 18ce54d4x5t
```

`scoped-timeline list` is list-only and returns promoted-only timeline entries for the account scope.

## Draft And Scheduled Tweets

```bash
agent-ads x draft-tweets list --account-id 18ce54d4x5t
agent-ads x draft-tweets get --account-id 18ce54d4x5t --draft-tweet-id dt123
agent-ads x scheduled-tweets list --account-id 18ce54d4x5t
agent-ads x scheduled-tweets get --account-id 18ce54d4x5t --scheduled-tweet-id st123
```

These commands inspect existing tweet assets only. They do not create, update, or schedule tweets.

## Media, Apps, And Cards

```bash
agent-ads x account-media list --account-id 18ce54d4x5t
agent-ads x media-library list --account-id 18ce54d4x5t
agent-ads x account-apps list --account-id 18ce54d4x5t
agent-ads x cards list --account-id 18ce54d4x5t
```

- `account-media` and `media-library` inspect stored assets available to the ads account.
- `account-apps` covers app objects referenced by app campaigns.
- `cards` exposes card creatives without adding write flows or preview helpers.

## Read-Only Boundary

The X creative surface in `agent-ads` intentionally excludes:

- tweet/card/media creation
- update or delete operations
- preview endpoints
- creative/tweet association mutations
