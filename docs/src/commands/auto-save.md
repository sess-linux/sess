# `sess auto-save`

```
sess auto-save <name> [--interval N]
sess auto-save <name> --stop
```

Starts (or stops) a background process that periodically re-captures and
re-saves `<name>` under itself, so recent state isn't lost if you forget
to run `sess save` yourself before closing the terminal.

```bash
sess auto-save backend-debug                # every [autosave] interval seconds (default 30)
sess auto-save backend-debug --interval 60  # override the interval for this one
sess auto-save backend-debug --stop         # stop it
```

The session must already be running (`sess auto-save` doesn't start one —
use `sess start <name> --auto-save` for that in one step).

The loop exits on its own once the tmux session it's watching disappears,
and it preserves any [persisted environment variables](../environment.md)
from the last manual `sess save` rather than dropping them.

See [Configuration](../configuration.md#autosave) to make this the default
for every new session instead of calling it separately.

**Known limitation:** the background process is a plain detached child,
not a fully daemonized one (no double-fork / `setsid`). Robust on most
Linux setups, but see [Limitations](../limitations.md).
