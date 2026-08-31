# `sess status`

```
sess status
```

A quick summary of every session's [state](../states.md): counts up top,
then one line per session with its state and (for stale/broken ones) why.

```
1 running, 2 saved, 1 stale, 0 broken (4 total)
  RUNNING  backend-debug
  SAVED    scraper-mimir
  SAVED    onboarding-call
  STALE    old-project — a saved directory no longer exists: /tmp/old-clone
```

Uses the exact same state detection as `list`, `switch`, and the picker —
nothing here is computed differently.
