# `sess delete`

```
sess delete <name>
```

Deletes a saved session's **snapshot only**. As of v0.3 this does **not**
touch a live tmux session with the same name — if one is running, `sess`
tells you so and points you at [`close`](./close.md):

```
saved session 'backend-debug' deleted.
note: 'backend-debug' is still running as a live tmux session — use `sess close backend-debug` to stop it too.
```

For the previous all-in-one behavior (kill the live session *and* delete
the snapshot), use [`kill-session`](./kill.md) instead — see the
[overlap table](../commands.md#save--open--switch--close--delete--kill-session-at-a-glance)
for the full picture of how these relate.
