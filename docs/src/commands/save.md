# `sess save`

```
sess save [name] [--force]
```

Saves the current tmux session (the one `sess` is being run from) under a
name. Defaults to the current session's own name if none is given.

```bash
sess save
sess save backend-debug-friday
sess save backend-debug-friday --force   # overwrite without prompting
```

If a saved session already exists under that name, `sess` asks for
confirmation before overwriting — unless `--force` is passed. `sess` never
overwrites silently.

If [`[environment] persist`](../environment.md) is configured, this is
also the point where allow-listed variables are captured (with a warning
for any that look like secrets).

Must be run from inside a tmux session; if it isn't, `sess` fails with a
clear message pointing you to `sess start`.

**Note:** if you run `sess save` from inside one of the panes it's saving,
that pane's "current command" at the instant of capture is `sess` itself
— `sess` detects this and treats it the same as an idle shell (nothing to
relaunch), rather than recording a self-referential command that would
re-run itself on restore.
