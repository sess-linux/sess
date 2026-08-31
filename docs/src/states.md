# Session states

Every session `sess` knows about — every saved snapshot, every live tmux
session, or both at once under the same name — is in exactly one of four
states. This is computed from real system state, not inferred separately
by each command: `list`, `status`, `switch`, and the picker all call the
same detection logic, so they never disagree with each other.

| State     | Meaning |
|-----------|---------|
| `RUNNING` | A live tmux session exists for this name. |
| `SAVED`   | A valid snapshot exists, nothing live, and everything it references still looks reachable. |
| `STALE`   | A valid snapshot exists, nothing live, but something it references — currently: a pane's working directory — no longer exists on disk. |
| `BROKEN`  | The saved file exists but could not be parsed (corrupted JSON, usually). |

`RUNNING` takes priority: if a live tmux session exists under that name,
that's the state, even if its saved snapshot happens to look stale or
broken underneath.

## Where you see this

```bash
sess list      # STATE column
sess status    # grouped counts + one line per session
sess switch    # color-coded in the picker, drives what Enter does
```

## What happens with each state in the picker

- **Running** or **Saved** — pressing Enter opens it (attaches or restores).
- **Stale** — pressing Enter shows what's missing and asks: open anyway,
  delete, or cancel. Nothing happens silently.
- **Broken** — pressing Enter shows the parse error. There's nothing to
  open; use `d` to delete it, or run [`sess prune`](./commands/prune.md)
  to clear out every broken snapshot at once.
