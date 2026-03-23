# X Audiences And Measurement

Use this guide when the task is audiences, suppression lists, conversion tags, app lists, or AB tests in X Ads.

## Audience And Suppression Surfaces

```bash
agent-ads x custom-audiences list --account-id 18ce54d4x5t
agent-ads x custom-audiences get --account-id 18ce54d4x5t --custom-audience-id ca123

agent-ads x do-not-reach-lists list --account-id 18ce54d4x5t
agent-ads x do-not-reach-lists get --account-id 18ce54d4x5t --do-not-reach-list-id dnr123
```

These map directly to X Ads audience objects. The CLI does not upload or mutate them.

## Measurement Surfaces

```bash
agent-ads x web-event-tags list --account-id 18ce54d4x5t
agent-ads x web-event-tags get --account-id 18ce54d4x5t --web-event-tag-id wet123

agent-ads x app-lists list --account-id 18ce54d4x5t
agent-ads x app-lists get --account-id 18ce54d4x5t --app-list-id al123

agent-ads x ab-tests list --account-id 18ce54d4x5t
agent-ads x ab-tests get --account-id 18ce54d4x5t --ab-test-id ab123
```

- `web-event-tags` covers web conversion tag inspection.
- `app-lists` covers mobile/app-list measurement inputs.
- `ab-tests` exposes read-only AB testing objects.

## What Not To Expect

- no audience uploads
- no suppression list edits
- no tag creation or placement helpers
- no experiment mutations
