# Limitations

- **Commands are relaunched, not resumed.** `sess` restarts each saved
  command from scratch — it does not preserve a process's internal state
  (variables loaded into a REPL, an in-progress download, and so on).
  Auto-save (v0.3) helps you lose less by saving more often, but it's not
  a substitute for real process checkpointing — see
  [Roadmap](./roadmap.md) for where that stands.
- **Linux only.** Resolving a pane's full command line (with arguments)
  relies on reading `/proc/<pid>/cmdline`.
- **Auto-save's background process isn't a full daemon.** It's a plain
  detached child (no double-fork / `setsid`), which is robust on most
  Linux setups but not guaranteed to survive every possible parent-exit
  scenario the way a proper daemon would. Kept simple on purpose — see
  [`auto-save`](./commands/auto-save.md).
- **No automatic retention policy yet.** `sess list` shows you how much
  each session weighs, and `sess prune` removes files that are outright
  corrupted, but nothing ages out saved sessions automatically just for
  being old or large. Cleanup of otherwise-valid sessions is still manual
  (`sess delete`).
- **Environment persistence is opt-in and flat.** `sess` doesn't do
  anything clever with variable scoping or per-window environments — it's
  a single allow-listed set of name/value pairs per session. See
  [Environment variable persistence](./environment.md).
