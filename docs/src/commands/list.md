# `sess list`

```
sess list [--json]
```

Lists every session `sess` knows about — the union of saved snapshots and
live tmux sessions — with its [state](../states.md), pane/window count, and
on-disk size, sorted by name.

```bash
sess list
```

```
NAME                 STATE     WIN PANE       SIZE  SAVED
backend-debug        RUNNING     1    3      633 B  2026-08-30 03:16
scraper-mimir        STALE       1    2      1.2 KB  2026-08-29 22:04
  a saved directory no longer exists: /tmp/old-clone
```

A `STALE` or `BROKEN` entry prints a second, indented line explaining why
— the same detail shown in the picker and in `sess status`.

Pass `--json` for the same data as JSON, for scripting:

```bash
sess list --json
```
