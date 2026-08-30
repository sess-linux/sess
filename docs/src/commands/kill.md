# `sess kill-session` / `kill-window` / `kill-server`

Thin, tmux-flavored wrappers for killing things directly, without leaving
`sess`:

```
sess kill-session <name>
sess kill-window <target>
sess kill-server
```

`kill-session` also deletes the matching saved snapshot, if one exists —
this is a destructive shortcut, not the same as `sess delete` followed by
walking away; double-check the name before running it.
