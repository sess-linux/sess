# `sess close`

```
sess close <name>
```

Kills the live tmux session but leaves its saved snapshot untouched — the
counterpart of [`open`](./open.md). `close` then `open` round-trips back
to the same state (modulo whatever changed while it was closed, obviously).

Fails with a clear message (naming the session's actual
[state](../states.md)) if `<name>` isn't currently running.

If you want to close *and* forget it in one step, use
[`kill-session`](./kill.md) instead.
