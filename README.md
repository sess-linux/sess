# sess

A small utility to save and restore tmux sessions without depending on tmux plugins.

`sess` snapshots a live tmux session, stores it locally, and recreates it later with its windows, panes, working directories, layout, and foreground commands.

## Why this exists

Tools like `tmux-resurrect` and `sesh` already solve similar problems, but they often depend on tmux plugin ecosystems or extra setup.

`sess` is a standalone binary: build it once, put it in your PATH, and use it directly on top of tmux.

## Requirements

- tmux installed and available in PATH
- Linux-tested because it reads `/proc` to recover the full foreground command from each pane

## Installation

```bash
cargo build --release
sudo cp target/release/sess /usr/local/bin/
```

## Quick start

```bash
# Start a new tmux session tracked by sess
sess start backend-debug

# Work normally inside tmux
# ...

# Save the current tmux session under a name
sess save

# Or save it with an explicit name
sess save backend-debug

# List saved sessions
sess list

# Restore it later
sess open backend-debug

# Delete a saved snapshot
sess delete backend-debug

# Interactive picker for opening and deleting saved sessions
sess
```

## Main commands

```bash
sess start <name>
    Start a new tmux session and track it with sess.

sess save [name] [--force]
    Save the current tmux session under a name. If the name already exists,
    it prompts unless --force is used.

sess list [--json]
    Show saved sessions in a table or JSON format.

sess open <name> [--force]
    Restore a saved session. If it already exists, attach by default unless --force is used.

sess attach <name>
    Attach to a tmux session that is already running.

sess rename <from> <to>
    Rename a saved snapshot.

sess duplicate <from> <to>
    Duplicate a saved snapshot under a new name.

sess status
    Show live tmux sessions and saved sess snapshots.

sess prune
    Remove invalid or corrupted saved session files.

sess delete <name>
    Delete a saved session and kill its live tmux session if it exists.

sess kill-session <name>
    Kill a tmux session by name.

sess kill-window <target>
    Kill a tmux window by target such as session:window.

sess kill-server
    Kill the entire tmux server.
```

## How it works

- Capture: `sess` reads the current tmux state, including windows, panes, directory, layout, and the foreground command running in each pane.
- Full command recovery: for each pane, it reads `/proc/<pid>/cmdline` to recover the real command line instead of only the binary name.
- Restore: it recreates windows and panes with `tmux new-session`, `new-window`, `split-window`, and reapplies the original layout using `select-layout`.
- Relaunch: it re-sends the stored commands to each pane so the session feels recreated rather than merely restored.
- Safety: `sess open` and `sess save` detect collisions and avoid destructive behavior unless forced.

## Storage

Saved snapshots are stored in:

```bash
~/.local/share/sess/
```

Each file is a JSON snapshot containing the layout and the commands associated with the session.

## Known limitations

- It does not restore the internal runtime state of a process (for example, an interactive REPL state). It re-launches commands from scratch.
- It is Linux-oriented because it relies on `/proc` for full command inspection.
- It is an MVP focused on practicality and speed, not full OS-level checkpointing.

## Project structure

```text
src/
├── core/
│   ├── model.rs     # snapshot data structures
│   ├── capture.rs   # tmux + /proc capture logic
│   ├── storage.rs   # save/load/list/delete logic
│   └── restore.rs   # recreate session in tmux
├── cli/
│   ├── mod.rs      # clap command definitions
│   └── commands.rs # CLI command implementations
├── tui/
│   └── mod.rs      # interactive picker UI
├── main.rs         # entry point
└── lib.rs          # not currently used
```

## License

This project is currently unlicensed unless you decide to add a license file for distribution.
