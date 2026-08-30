use serde::{Deserialize, Serialize};

/// A single pane inside a window.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaneSnapshot {
    pub index: u32,
    pub cwd: String,
    /// Command that was running in the pane (e.g. "vim src/main.rs", "npm run dev").
    /// Empty if the pane only had an interactive shell without any foreground command.
    pub command: String,
}

/// A window (can contain multiple panes).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowSnapshot {
    pub index: u32,
    pub name: String,
    /// Native tmux layout string (e.g. "a1b2,80x24,0,0,3").
    /// Enables exact recreation of split geometry with `select-layout`.
    pub layout: String,
    pub panes: Vec<PaneSnapshot>,
}

/// A complete saved session.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionSnapshot {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub windows: Vec<WindowSnapshot>,
}

/// Short metadata used to list sessions without loading the full JSON snapshot.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionSummary {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub pane_count: usize,
    /// Size on disk in bytes (snapshot.json + any future --freeze data).
    pub size_bytes: u64,
}
