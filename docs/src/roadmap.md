# Roadmap

- **Retention policy / automatic `gc`.** Time-decaying snapshot retention
  (recent saves kept in full, older ones thinned automatically), plus caps
  configurable in a `.sessconfig`.
- **Experimental process checkpointing (CRIU).** An opt-in `--freeze` mode
  that attempts to preserve real process state on save and restore instead
  of relaunching from scratch, falling back automatically to the normal
  relaunch behavior when it can't restore cleanly. Being evaluated for
  viability before committing to it — see the project's design notes.
