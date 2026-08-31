# Commands

| Command | Purpose |
|---|---|
| [`start`](./commands/start.md) | Start a new session tracked by `sess` |
| [`save`](./commands/save.md) | Save the current tmux session under a name |
| [`list`](./commands/list.md) | List sessions with their state, size, and pane count |
| [`open`](./commands/open.md) | Restore (or attach to) a saved session |
| [`switch`](./commands/switch.md) | Open the interactive picker |
| [`attach`](./commands/attach.md) | Attach to a tmux session already running |
| [`close`](./commands/close.md) | Close a running session, keep its snapshot |
| [`delete`](./commands/delete.md) | Delete a saved snapshot only |
| [`rename`](./commands/rename.md) | Rename a saved session |
| [`duplicate`](./commands/duplicate.md) | Duplicate a saved session under a new name |
| [`status`](./commands/status.md) | Show every session's state at a glance |
| [`doctor`](./commands/doctor.md) | Check the health of the installation and storage |
| [`auto-save`](./commands/auto-save.md) | Start/stop background auto-save for a session |
| [`prune`](./commands/prune.md) | Remove corrupt or invalid saved session files |
| [`kill-session` / `kill-window` / `kill-server`](./commands/kill.md) | tmux-style kill commands |
| [Picker](./commands/picker.md) | Interactive TUI, run with no arguments |

Every command also responds to `-h` / `--help` for a quick reminder.

## `save` / `open` / `switch` / `close` / `delete` / `kill-session`, at a glance

These six overlap in what they touch, so here's the full picture in one
place:

| Command | Touches the live tmux session? | Touches the saved snapshot? |
|---|---|---|
| `save` | reads | creates/updates |
| `open` | creates | reads |
| `switch` | creates or reads, depending on state | reads |
| `close` | kills | — |
| `delete` | — | removes |
| `kill-session` | kills | removes |

`close` then `delete` is the deliberate two-step way to fully tear down a
session; `kill-session` is the one-shot shortcut for the same end result.
