# `sess open`

```
sess open <name> [--force]
```

Reopens a saved session. If a live tmux session with that name is already
running, `sess` attaches to it instead of restoring — your saved snapshot
is left untouched.

```bash
sess open backend-debug
```

Pass `--force` to kill the live session first and restore fresh from the
saved snapshot instead of attaching to what's currently running.

```bash
sess open backend-debug --force
```

If the snapshot has any [persisted environment variables](../environment.md),
`sess` prints a one-line summary of what it restored. Selecting a session
by state (running/saved/stale/broken) instead of by name directly? See
[`switch`](./switch.md) and [Session states](../states.md).
