# Restoring a session

`sess open` rebuilds the session window by window:

1. `tmux new-session` for the first window, in the first pane's saved
   working directory.
2. `tmux split-window` once per additional saved pane.
3. `tmux select-layout` with the exact layout string captured at save
   time, so the geometry matches precisely instead of an even/generic
   split.
4. `tmux send-keys` to relaunch each pane's saved command — guarded with
   `--` so a command that happens to start with `-` is never misread as a
   flag.

If a live tmux session with the target name already exists, `sess` refuses
to restore over it and attaches instead (see [`open`](../commands/open.md)
for the `--force` override).

Restoring relaunches commands from scratch — it does not preserve a
process's internal memory state (variables loaded in a REPL, for example).
See [Limitations](../limitations.md).
