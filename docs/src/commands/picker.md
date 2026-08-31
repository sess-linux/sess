# Picker (interactive TUI)

```
sess
sess switch   # identical — see switch.md
```

Shows every session `sess` knows about with its
[state](../states.md), pane count, a relative size bar, and its exact
on-disk size, plus a running total at the top.

## Keys

| Key | Action |
|---|---|
| `↑` / `k`, `↓` / `j` | Move selection |
| `Enter` | State-dependent — see below |
| `d` | Delete the selected saved snapshot |
| `r` | Rename (type the new name, `Enter` to confirm, `Esc` to cancel) |
| `c` | Clone under a new name (same prompt as rename) |
| `s` | Save the tmux session `sess` is currently attached from (not necessarily the selected row) |
| `q` / `Esc` | Quit |

Rename, clone, delete, and save all happen in place — the picker stays
open and the list refreshes, so you can chain several actions without
leaving it. Only opening a session exits the picker (attaching hands the
terminal over to tmux).

## What `Enter` does, by state

- **Running** or **Saved** — attaches or restores immediately.
- **Stale** — shows what's missing and asks: `[o]pen anyway`, `[d]elete`,
  or `[c]ancel`. Nothing happens until you choose.
- **Broken** — shows the parse error; press any key to dismiss. There's
  nothing to open — use `d` to delete it.
