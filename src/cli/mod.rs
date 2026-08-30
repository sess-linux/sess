pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sess", version, about = "Save and reopen terminal sessions, standalone.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Starts a new session tracked by sess (no need to already be inside tmux).
    Start {
        name: String,
    },
    /// Saves the current tmux session under a name.
    Save {
        /// Name used to save it (defaults to the current tmux session name).
        name: Option<String>,
        /// Overwrites without prompting if a session with that name already exists.
        #[arg(long)]
        force: bool,
    },
    /// Lists saved sessions, including the number of panes and their size.
    List {
        /// Emits the list as JSON instead of the human-friendly table.
        #[arg(long)]
        json: bool,
    },
    /// Reopens (or attaches to) a saved session.
    Open {
        name: String,
        /// Restores even if a tmux session with the same name is already running.
        #[arg(long)]
        force: bool,
    },
    /// Attaches to a tmux session that is already running.
    Attach {
        name: String,
    },
    /// Deletes a saved session.
    Delete {
        name: String,
    },
    /// Renames a saved session.
    Rename {
        from: String,
        to: String,
    },
    /// Duplicates a saved session under a new name.
    Duplicate {
        from: String,
        to: String,
    },
    /// Displays the current tmux and saved-sessions status.
    Status,
    /// Prunes corrupt or invalid saved session files.
    Prune,
    /// Kills a tmux session by name (tmux-like `kill-session`).
    KillSession {
        name: String,
    },
    /// Kills a tmux window by target (tmux-like `kill-window`).
    KillWindow {
        target: String,
    },
    /// Kills the entire tmux server (tmux-like `kill-server`).
    KillServer,
}
