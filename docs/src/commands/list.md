# `sess list`

```
sess list [--json]
```

Lists every saved session with its pane count, on-disk size, and when it
was saved, newest first.

```bash
sess list
```

```
NAME                   PANES       SIZE  SAVED
backend-debug              3      312 B  2026-08-30 03:16
scraper-mimir               4      1.2 KB  2026-08-29 22:04
```

Pass `--json` to get the same data as JSON, for scripting:

```bash
sess list --json
```
