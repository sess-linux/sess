# Roadmap

Auto-save, environment persistence, session states, and `sess doctor`
shipped in v0.3. What's still open:

- **Retention policy / automatic `gc`.** Time-decaying snapshot retention
  (recent saves kept in full, older ones thinned automatically), plus caps
  configurable in `config.toml`. `sess prune` today only removes files
  that fail to parse — it doesn't do age- or size-based cleanup.
- **Experimental process checkpointing (CRIU).** A genuinely different
  mechanism from auto-save: instead of relaunching a saved command from
  scratch, actually freeze and later resume the real process, memory
  included. Investigated for viability — CRIU itself is mature technology
  (used for container live-migration and HPC job preemption) and handles
  pseudo-terminals well, but it requires root/sudo each time, and needs
  matching system library versions between save and restore, which makes
  it unreliable across anything longer than "pause for a bit before a
  reboot." If it ships, it would be strictly opt-in (`--freeze`) with
  automatic fallback to the normal relaunch behavior whenever it can't
  restore cleanly — never the default.
