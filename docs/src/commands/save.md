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
confirmation before overwriting — unless `--force` is passed. This is
deliberate: `sess` never overwrites silently.

Must be run from inside a tmux session; if it isn't, `sess` fails with a
clear message pointing you to `sess start`.
