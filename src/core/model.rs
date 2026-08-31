use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Current on-disk snapshot format version. Bump this and add a migration step
/// in `storage::load` whenever a change would otherwise break older snapshots.
pub const CURRENT_SNAPSHOT_VERSION: u32 = 2;

fn default_version() -> u32 {
    // Snapshots written before this field existed are implicitly version 1.
    1
}

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
    #[serde(default = "default_version")]
    pub version: u32,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub windows: Vec<WindowSnapshot>,
    /// Explicitly persisted environment variables (only those allow-listed in
    /// config, never captured automatically). Absent in snapshots saved before
    /// this feature existed, hence the default.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

impl SessionSnapshot {
    pub fn new(name: String, windows: Vec<WindowSnapshot>, env: BTreeMap<String, String>) -> Self {
        Self {
            version: CURRENT_SNAPSHOT_VERSION,
            name,
            created_at: chrono::Local::now(),
            windows,
            env,
        }
    }

    pub fn pane_count(&self) -> usize {
        self.windows.iter().map(|w| w.panes.len()).sum()
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

/// Short metadata used to list sessions without loading the full JSON snapshot.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionSummary {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub pane_count: usize,
    pub window_count: usize,
    /// Size on disk in bytes (snapshot.json + any future --freeze data).
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_snapshot_without_version_or_env_still_parses() {
        // Exactly the shape written by versions before `version`/`env` existed.
        let old_json = r#"{
            "name": "legacy",
            "created_at": "2026-01-01T00:00:00-00:00",
            "windows": []
        }"#;
        let snapshot: SessionSnapshot =
            serde_json::from_str(old_json).expect("old snapshots must still parse");
        assert_eq!(snapshot.version, 1);
        assert!(snapshot.env.is_empty());
    }

    #[test]
    fn new_snapshot_round_trips_through_json() {
        let mut env = BTreeMap::new();
        env.insert("NODE_ENV".to_string(), "production".to_string());

        let original = SessionSnapshot::new(
            "roundtrip".into(),
            vec![WindowSnapshot {
                index: 0,
                name: "w".into(),
                layout: "layout-string".into(),
                panes: vec![PaneSnapshot {
                    index: 0,
                    cwd: "/tmp".into(),
                    command: "top".into(),
                }],
            }],
            env,
        );

        let json = serde_json::to_string(&original).unwrap();
        let parsed: SessionSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, CURRENT_SNAPSHOT_VERSION);
        assert_eq!(parsed.pane_count(), 1);
        assert_eq!(parsed.window_count(), 1);
        assert_eq!(parsed.env.get("NODE_ENV"), Some(&"production".to_string()));
    }
}
