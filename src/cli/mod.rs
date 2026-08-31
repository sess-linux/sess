pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "sess",
    version,
    about = "Save and reopen terminal sessions, standalone."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Starts a new session tracked by sess (no need to already be inside tmux).
    Start {
        name: String,
        /// Also starts a background auto-save loop for this session
        /// (overrides config either way, see --no-auto-save).
        #[arg(long, conflicts_with = "no_auto_save")]
        auto_save: bool,
        /// Explicitly skips auto-save even if config.toml has it enabled by default.
        #[arg(long)]
        no_auto_save: bool,
    },
    /// Saves the current tmux session under a name (creates or updates the snapshot).
    Save {
        /// Name used to save it (defaults to the current tmux session name).
        name: Option<String>,
        /// Overwrites without prompting if a session with that name already exists.
        #[arg(long)]
        force: bool,
    },
    /// Lists saved and running sessions, with their state, size, and pane count.
    List {
        /// Emits the list as JSON instead of the human-friendly table.
        #[arg(long)]
        json: bool,
    },
    /// Restores and opens a saved session (or attaches, if it's already running).
    Open {
        name: String,
        /// Restores from the snapshot even if a tmux session with the same name is already running.
        #[arg(long)]
        force: bool,
    },
    /// Opens the interactive session switcher (same as running `sess` with no arguments).
    Switch,
    /// Attaches to a tmux session that is already running.
    Attach { name: String },
    /// Closes a running session without deleting its saved snapshot.
    Close { name: String },
    /// Deletes a saved session's snapshot. Does not touch a live tmux session —
    /// use `sess close` first (or `sess kill-session` to do both at once).
    Delete { name: String },
    /// Renames a saved session.
    Rename { from: String, to: String },
    /// Duplicates a saved session under a new name.
    Duplicate { from: String, to: String },
    /// Displays the current state of every session sess knows about.
    Status,
    /// Checks the health of the sess installation, storage, and saved sessions.
    Doctor {
        /// Applies safe, non-destructive fixes (never deletes data).
        #[arg(long)]
        fix: bool,
    },
    /// Starts or stops the background auto-save loop for a session.
    AutoSave {
        name: String,
        /// Stops the auto-save loop instead of starting it.
        #[arg(long)]
        stop: bool,
        /// Save interval in seconds (defaults to the configured value, or 30s).
        #[arg(long)]
        interval: Option<u64>,
    },
    /// Prunes corrupt or invalid saved session files.
    Prune,
    /// Kills a tmux session by name and deletes its saved snapshot, if any
    /// (tmux-like `kill-session`, but sess-aware — see also `close` and `delete`
    /// for the two halves of this on their own).
    KillSession { name: String },
    /// Kills a tmux window by target (tmux-like `kill-window`).
    KillWindow { target: String },
    /// Kills the entire tmux server (tmux-like `kill-server`).
    KillServer,
    /// Internal: runs the auto-save loop body. Not meant to be invoked directly.
    #[command(hide = true, name = "__autosave-loop")]
    AutoSaveLoop { name: String, interval: u64 },
}
