# `sess delete`

```
sess delete <name>
```

Deletes a saved session. If a live tmux session with the same name is
running, it's killed too. Safe to run even if only one of the two (saved
snapshot or live session) exists.
