# Capturing a session

When you run `sess save`, it reads the current tmux session's structure via
`tmux list-windows` and `tmux list-panes`, collecting for each pane:

- its working directory (`pane_current_path`)
- the exact layout string tmux uses internally (`window_layout`), which is
  later fed straight into `select-layout` to reproduce the geometry exactly
- what's running in it

## Full command lines, not just process names

tmux's `pane_current_command` only reports a process's short name — `sleep`,
not `sleep 300`. Restoring with just the name loses all arguments, so
`sess` resolves the real, full command line by reading `/proc/<pid>/cmdline`
for the process actually running in the foreground of that pane (found by
walking the tmux pane's shell process's children until one matches the name
tmux already identified). This only works on Linux.

Idle shells (`bash`, `zsh`, `fish`, `sh`, `dash` with nothing running) are
recorded as empty — there's nothing to relaunch, `sess` just returns the
cursor to that directory on restore.
