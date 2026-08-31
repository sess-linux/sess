# `sess kill-session` / `kill-window` / `kill-server`

Thin, tmux-flavored wrappers for killing things directly, without leaving
`sess`:

```
sess kill-session <name>
sess kill-window <target>
sess kill-server
```

`kill-session` also deletes the matching saved snapshot, if one exists —
this is the one-shot equivalent of [`close`](./close.md) followed by
[`delete`](./delete.md), not the same as either on its own. Double-check
the name before running it; see the
[overlap table](../commands.md#save--open--switch--close--delete--kill-session-at-a-glance)
if you want the exact difference from `close`/`delete` spelled out.
