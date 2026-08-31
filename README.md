# sess

A small utility to save and restore tmux sessions without depending on tmux
plugins.

`sess` snapshots a live tmux session — its windows, panes, working
directories, layout, and foreground commands — stores it locally, and
recreates it later, even after a reboot.

## Why this exists

Tools like `tmux-resurrect` and `sesh` already solve similar problems, but
they depend on the tmux plugin ecosystem (TPM and friends) being already set
up. `sess` is a standalone binary: build it once, put it on your `PATH`, and
use it directly on top of tmux — no plugins required.

## Requirements

- tmux installed and available on `PATH`
- Linux (it reads `/proc/<pid>/cmdline` to recover the full foreground
  command of each pane — see [How it works](#how-it-works))

## Installation

From a release binary:

```bash
tar -xzf sess-VERSION-linux-x86_64.tar.gz
sudo mv sess /usr/local/bin/
```

From source:

```bash
cargo build --release
sudo cp target/release/sess /usr/local/bin/
```

## Quick start

```bash
# Start a new tmux session tracked by sess
sess start backend-debug

# Work normally inside tmux ...

# Save it under a name (defaults to the current session's own name)
sess save

# Later — tomorrow, after a reboot, whenever
sess open backend-debug

# Or just browse everything you've saved
sess
```

## Sessions have a state

Every session sess knows about is one of four states, used consistently by
`list`, `status`, `switch`, and the picker — nothing is duplicated or
inferred differently in different places:

| State     | Meaning                                                              |
|-----------|-----------------------------------------------------------------------|
| `RUNNING` | A live tmux session exists for this name.                            |
| `SAVED`   | A valid snapshot exists, nothing live, everything it references still looks reachable. |
| `STALE`   | A valid snapshot exists, but something it references (e.g. a pane's directory) no longer exists. |
| `BROKEN`  | The saved file exists but could not be parsed.                       |

## Commands

```text
sess start <name> [--auto-save] [--no-auto-save]
    Start a new tmux session tracked by sess. --auto-save also starts a
    background loop that periodically re-saves it (see Auto-save below).
    --no-auto-save opts out even if config.toml enables it by default.

sess save [name] [--force]
    Save the current tmux session under a name (creates or updates the
    snapshot). Defaults to the current session's own name.

sess list [--json]
    List every session sess knows about, with its state, size, and pane count.

sess open <name> [--force]
    Restore a saved session. Attaches instead if it's already running,
    unless --force is used to restore fresh over it.

sess switch
    Open the interactive picker (same as running `sess` with no arguments).

sess attach <name>
    Attach to a tmux session that is already running.

sess close <name>
    Close a running session without deleting its saved snapshot.

sess delete <name>
    Delete a saved snapshot. Does not touch a live tmux session with the
    same name — use `close` for that, or `kill-session` to do both at once.

sess rename <from> <to>
sess duplicate <from> <to>

sess status
    Quick summary of every session's state.

sess doctor [--fix]
    Check the health of the sess installation, storage, and saved sessions.
    --fix applies only safe, non-destructive repairs.

sess auto-save <name> [--stop] [--interval N]
    Start (or stop) a background auto-save loop for a running session.

sess prune
    Remove saved session files that are corrupted and can't be read.

sess kill-session <name>
sess kill-window <target>
sess kill-server
    tmux-flavored kill commands. kill-session also removes the matching
    saved snapshot, if any — a destructive shortcut, not the same as
    `close` followed by `delete`.
```

## Auto-save

```bash
sess start backend-debug --auto-save
# or, for a session already running:
sess auto-save backend-debug
sess auto-save backend-debug --stop
```

Spawns a background process that periodically re-captures and re-saves the
session under itself, so you don't lose recent state if you forget to
`sess save` before closing the terminal. Configurable via
`~/.config/sess/config.toml`:

```toml
[autosave]
enabled = true
interval = 30
```

With `enabled = true`, every `sess start <name>` turns auto-save on by
default — no need to pass `--auto-save` each time. Use `sess start <name>
--no-auto-save` to opt out for one particular session.

The background loop exits on its own once the tmux session it's watching
disappears. It is a plain detached child process, not a fully daemonized
one (no double-fork/`setsid`) — simple by design, see
[Known limitations](#known-limitations).

## Environment variable persistence

By default, `sess` never captures environment variables. You opt in
explicitly, by name, in `~/.config/sess/config.toml`:

```toml
[environment]
persist = ["NODE_ENV", "EDITOR", "PROJECT_ENV"]
```

Only variables listed here are ever saved, and only if they're actually set
at save time. Variable names that look like they might hold a secret
(`*_KEY`, `*_TOKEN`, `*_SECRET`, `PASSWORD`, `AUTH`, `CREDENTIAL`) still get
persisted if you explicitly list them — `sess` isn't a secrets manager, it
just won't do it by accident — but it prints a warning on save so you notice.

## How it works

- **Capture**: reads the current tmux session's windows, panes, working
  directories, and tmux's own layout string (fed straight into
  `select-layout` on restore, for exact geometry).
- **Full command recovery**: tmux only exposes a pane's process *name*
  (`sleep`), not its full command line (`sleep 300`). `sess` resolves the
  real command by reading `/proc/<pid>/cmdline` for the process actually
  running in the pane's foreground.
- **Restore**: recreates windows and panes with `tmux new-session` /
  `split-window`, reapplies the exact saved layout, restores any persisted
  environment variables via `export`, then relaunches each pane's saved
  command.
- **Safety**: `sess open` and `sess save` detect collisions and never
  overwrite or restore over something live without `--force`; `delete`
  never touches a live tmux session.

## Storage

Saved snapshots live at `~/.local/share/sess/<name>.json` — plain JSON, no
database:

```json
{
  "version": 2,
  "name": "backend-debug",
  "created_at": "2026-08-30T03:16:25Z",
  "windows": [ ... ],
  "env": { "NODE_ENV": "production" }
}
```

Snapshots saved by earlier versions of `sess` (without `version` or `env`)
still load fine — those fields default sensibly on read.

## Known limitations

- Commands are relaunched, not resumed — a process's internal state (a
  REPL's loaded variables, an in-progress download) is not preserved.
- Linux only — full command-line recovery relies on `/proc`.
- Auto-save's background process is a simple detached child, not a full
  daemon; robust on most Linux setups, but not guaranteed to survive every
  possible parent-exit scenario.
- No automatic retention policy yet — `sess list` shows you what each
  session weighs, but cleanup is manual (`sess delete` / `sess prune`).

## Project structure

```text
src/
├── core/
│   ├── model.rs     # snapshot data structures (versioned, backward compatible)
│   ├── capture.rs   # tmux + /proc capture logic
│   ├── storage.rs   # save/load/list/rename/prune logic (single source of truth for raw reads)
│   ├── restore.rs   # recreate a session in tmux, restore env vars
│   ├── state.rs     # RUNNING/SAVED/STALE/BROKEN — the shared state model
│   ├── config.rs    # ~/.config/sess/config.toml
│   ├── env.rs       # env var allow-listing + secret-name detection
│   ├── autosave.rs  # background auto-save loop
│   └── doctor.rs    # health checks
├── cli/
│   ├── mod.rs       # clap command definitions
│   └── commands.rs  # CLI command implementations
├── tui/
│   └── mod.rs       # interactive picker (states, rename/clone/save actions)
└── main.rs          # entry point

tests/
└── cli.rs           # integration tests against the built binary
```

## Documentation

Full command reference and internals: see [`docs/`](docs/) (mdBook —
`cd docs && mdbook serve`).

## Development

```bash
cargo fmt
cargo check
cargo test
cargo clippy
cargo build --release
```

## License

MIT — see [LICENSE](LICENSE).
