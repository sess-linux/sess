# `sess start`

```
sess start <name>
```

Starts a new tmux session tracked by `sess`, without needing to already be
inside tmux. If a live tmux session with that name already exists, `sess`
attaches to it instead of creating a duplicate.

```bash
sess start backend-debug
```
