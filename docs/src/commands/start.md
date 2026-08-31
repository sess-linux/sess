# `sess start`

```
sess start <name> [--auto-save] [--no-auto-save]
```

Starts a new tmux session tracked by `sess`, without needing to already be
inside tmux. If a live tmux session with that name already exists, `sess`
attaches to it instead of creating a duplicate.

```bash
sess start backend-debug
```

## Auto-save

```bash
sess start backend-debug --auto-save      # force it on for this session
sess start backend-debug --no-auto-save   # force it off for this session
sess start backend-debug                  # follows [autosave] enabled in config.toml
```

`--no-auto-save` always wins if both are somehow relevant; otherwise
`--auto-save` wins; otherwise the
[configured default](../configuration.md#autosave) applies. See
[`auto-save`](./auto-save.md) for what the background loop actually does.
