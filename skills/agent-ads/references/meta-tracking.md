# Meta Tracking

Use this file for custom conversions, pixels, dataset quality, and combined measurement-health checks.

## Commands

```bash
agent-ads meta custom-conversions list --account <account-id>
agent-ads meta pixels list --account <account-id>
agent-ads meta datasets get --account <account-id>
agent-ads meta pixel-health get --pixel <pixel-id>
```

## Notes

- `pixel-health` is a combined diagnostic view, not a raw EMQ passthrough.
- Use `datasets get` when the user needs dataset quality state across connected measurement surfaces.
- Use raw pixel and custom-conversion commands when the user wants the underlying objects rather than the health summary.
