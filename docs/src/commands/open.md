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
