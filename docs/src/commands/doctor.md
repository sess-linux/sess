# `sess doctor`

```
sess doctor [--fix]
```

Runs a fixed set of health checks and prints one line per check:

```
✓ tmux installed
✓ tmux version (tmux 3.4)
✓ storage directory (/home/user/.local/share/sess)
✓ storage writable
✓ session data valid
✓ terminal environment
```

- **✓** — fine.
- **⚠** — worth knowing about, but not blocking (e.g. some saved sessions
  are [stale](../states.md)).
- **✗** — something is actually broken (e.g. tmux isn't installed, or a
  saved session file can't be parsed).

Exits with status `1` if any check reports **✗**, `0` otherwise — safe to
use in a script.

## `--fix`

```bash
sess doctor --fix
```

Applies only safe, non-destructive repairs — currently, (re)creating the
storage directory if it's missing. `doctor --fix` never deletes data; use
[`prune`](./prune.md) explicitly for that.
