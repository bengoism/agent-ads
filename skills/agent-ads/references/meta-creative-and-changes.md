# Meta Creative And Changes

Use this file for creative inspection and forensic change history.

## Creative Lookups

```bash
agent-ads meta creatives get --id <creative-id>
agent-ads meta creatives preview --ad <ad-id>
```

## Activities

```bash
agent-ads meta activities list --account <account-id> --all
```

## Notes

- Use creative previews when the user needs the rendered ad payload, not just ad metadata.
- Use activities when the question is "what changed?" rather than "what performed badly?"
- Keep `--all` or `--max-items` explicit for activities because forensic investigations often need more than one page.
