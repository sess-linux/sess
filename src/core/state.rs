use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;

use super::capture;
use super::model::SessionSnapshot;
use super::storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StateKind {
    /// A live tmux session exists for this name.
    Running,
    /// A valid snapshot exists, no live session, and everything it references
    /// still looks reachable.
    Saved,
    /// A valid snapshot exists, no live session, but something it references
    /// (currently: a pane's working directory) no longer exists.
    Stale,
    /// The saved session file exists but could not be parsed.
    Broken,
}

impl std::fmt::Display for StateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            StateKind::Running => "RUNNING",
            StateKind::Saved => "SAVED",
            StateKind::Stale => "STALE",
            StateKind::Broken => "BROKEN",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionState {
    pub name: String,
    pub kind: StateKind,
    pub window_count: Option<usize>,
    pub pane_count: Option<usize>,
    pub size_bytes: Option<u64>,
    pub saved_at: Option<chrono::DateTime<chrono::Local>>,
    /// Human-readable explanation, populated for Stale and Broken.
    pub detail: Option<String>,
}

/// Computes the state of every session sess knows about: the union of saved
/// snapshots (valid or not) and live tmux sessions. This is the one place
/// that classifies sessions — `list`, `status`, `switch`, and the TUI all
/// call this instead of re-implementing the detection.
pub fn all() -> Result<Vec<SessionState>> {
    let raw = storage::list_all_raw()?;
    let live: BTreeSet<String> = capture::list_live_sessions()?.into_iter().collect();

    let mut names: BTreeSet<String> = live.clone();
    for entry in &raw {
        names.insert(entry.name.clone());
    }

    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let is_live = live.contains(&name);
        let entry = raw.iter().find(|e| e.name == name);

        let state = match entry {
            None => SessionState {
                name,
                kind: StateKind::Running,
                window_count: None,
                pane_count: None,
                size_bytes: None,
                saved_at: None,
                detail: None,
            },
            Some(entry) => match &entry.snapshot {
                Err(parse_error) => SessionState {
                    name,
                    kind: StateKind::Broken,
                    window_count: None,
                    pane_count: None,
                    size_bytes: Some(entry.size_bytes),
                    saved_at: None,
                    detail: Some(format!("saved file could not be read: {parse_error}")),
                },
                Ok(snapshot) => build_state(name, snapshot, entry.size_bytes, is_live),
            },
        };

        out.push(state);
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn of(name: &str) -> Result<Option<SessionState>> {
    Ok(all()?.into_iter().find(|s| s.name == name))
}

fn build_state(
    name: String,
    snapshot: &SessionSnapshot,
    size_bytes: u64,
    is_live: bool,
) -> SessionState {
    let window_count = Some(snapshot.window_count());
    let pane_count = Some(snapshot.pane_count());
    let saved_at = Some(snapshot.created_at);

    if is_live {
        return SessionState {
            name,
            kind: StateKind::Running,
            window_count,
            pane_count,
            size_bytes: Some(size_bytes),
            saved_at,
            detail: None,
        };
    }

    if let Some(missing) = first_missing_cwd(snapshot) {
        return SessionState {
            name,
            kind: StateKind::Stale,
            window_count,
            pane_count,
            size_bytes: Some(size_bytes),
            saved_at,
            detail: Some(format!("a saved directory no longer exists: {missing}")),
        };
    }

    SessionState {
        name,
        kind: StateKind::Saved,
        window_count,
        pane_count,
        size_bytes: Some(size_bytes),
        saved_at,
        detail: None,
    }
}

fn first_missing_cwd(snapshot: &SessionSnapshot) -> Option<String> {
    for window in &snapshot.windows {
        for pane in &window.panes {
            if !std::path::Path::new(&pane.cwd).exists() {
                return Some(pane.cwd.clone());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{PaneSnapshot, WindowSnapshot};

    fn snapshot_with_cwd(cwd: &str) -> SessionSnapshot {
        SessionSnapshot::new(
            "t".into(),
            vec![WindowSnapshot {
                index: 0,
                name: "w".into(),
                layout: String::new(),
                panes: vec![PaneSnapshot {
                    index: 0,
                    cwd: cwd.into(),
                    command: String::new(),
                }],
            }],
            Default::default(),
        )
    }

    #[test]
    fn saved_when_cwd_exists_and_not_live() {
        let snap = snapshot_with_cwd("/tmp");
        let state = build_state("t".into(), &snap, 10, false);
        assert_eq!(state.kind, StateKind::Saved);
    }

    #[test]
    fn stale_when_cwd_missing_and_not_live() {
        let snap = snapshot_with_cwd("/this/does/not/exist/sess-test");
        let state = build_state("t".into(), &snap, 10, false);
        assert_eq!(state.kind, StateKind::Stale);
        assert!(state.detail.is_some());
    }

    #[test]
    fn running_takes_priority_over_stale() {
        let snap = snapshot_with_cwd("/this/does/not/exist/sess-test");
        let state = build_state("t".into(), &snap, 10, true);
        assert_eq!(state.kind, StateKind::Running);
    }
}
