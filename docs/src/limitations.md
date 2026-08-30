# Limitations

- **Commands are relaunched, not resumed.** `sess` restarts each saved
  command from scratch — it does not preserve a process's internal state
  (variables loaded into a REPL, an in-progress download, and so on).
  Preserving actual process state would require checkpointing (e.g. CRIU),
  which is being evaluated separately — see [Roadmap](./roadmap.md).
- **Linux only.** Resolving a pane's full command line (with arguments)
  relies on reading `/proc/<pid>/cmdline`.
- **No retention policy yet.** `sess list` shows you how much each session
  weighs, but cleaning up old ones is manual (`sess delete`) — nothing ages
  out automatically.
