use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

use super::model::{SessionSnapshot, SessionSummary};

/// ~/.local/share/sess/
pub fn sessions_dir() -> Result<PathBuf> {
    let base = dirs::data_local_dir().context("could not resolve the local data directory")?;
    let dir = base.join("sess");
    fs::create_dir_all(&dir).context("could not create the sess storage directory")?;
    Ok(dir)
}

fn session_file(name: &str) -> Result<PathBuf> {
    Ok(sessions_dir()?.join(format!("{name}.json")))
}

pub fn exists(name: &str) -> Result<bool> {
    Ok(session_file(name)?.exists())
}

pub fn save(snapshot: &SessionSnapshot, overwrite: bool) -> Result<()> {
    let path = session_file(&snapshot.name)?;
    if path.exists() && !overwrite {
        bail!(
            "a saved session named '{}' already exists. Use --force to overwrite it.",
            snapshot.name
        );
    }
    let json = serde_json::to_string_pretty(snapshot)?;
    fs::write(&path, json).with_context(|| format!("could not write {}", path.display()))?;
    Ok(())
}

pub fn load(name: &str) -> Result<SessionSnapshot> {
    let path = session_file(name)?;
    if !path.exists() {
        bail!("no saved session named '{name}' exists");
    }
    let raw = fs::read_to_string(&path)?;
    let snapshot: SessionSnapshot = serde_json::from_str(&raw)
        .with_context(|| format!("the session file '{name}' is corrupted"))?;
    Ok(snapshot)
}

pub fn delete(name: &str) -> Result<()> {
    let path = session_file(name)?;
    if !path.exists() {
        bail!("no saved session named '{name}' exists");
    }
    fs::remove_file(&path)?;
    Ok(())
}

pub fn rename(from: &str, to: &str) -> Result<()> {
    let from_path = session_file(from)?;
    let to_path = session_file(to)?;

    if !from_path.exists() {
        bail!("no saved session named '{from}' exists");
    }
    if to_path.exists() {
        bail!("a saved session named '{to}' already exists");
    }

    let mut snapshot: SessionSnapshot = serde_json::from_str(&fs::read_to_string(&from_path)?)
        .with_context(|| format!("the session file '{from}' is corrupted"))?;
    snapshot.name = to.to_string();
    let json = serde_json::to_string_pretty(&snapshot)?;
    fs::write(&to_path, json).with_context(|| format!("could not write {}", to_path.display()))?;
    fs::remove_file(&from_path)?;
    Ok(())
}

/// One entry per `.json` file in the storage directory: the name (from the
/// filename, since a corrupted file may not have a readable `name` field), its
/// size on disk, and either the parsed snapshot or the parse error as text.
///
/// This is the single place that reads raw session files from disk — `list`,
/// `prune`, and `state::all` all build on top of this instead of re-reading
/// and re-parsing independently.
pub struct RawEntry {
    pub name: String,
    pub size_bytes: u64,
    pub snapshot: std::result::Result<SessionSnapshot, String>,
}

pub fn list_all_raw() -> Result<Vec<RawEntry>> {
    let dir = sessions_dir()?;
    let mut out = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);

        let snapshot = match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str::<SessionSnapshot>(&raw).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };

        out.push(RawEntry {
            name,
            size_bytes,
            snapshot,
        });
    }

    Ok(out)
}

pub fn delete_raw_file(name: &str) -> Result<()> {
    let path = session_file(name)?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

/// Removes any saved session file that fails to parse. Returns how many were
/// removed. This only ever deletes files that are already unusable — it never
/// touches a snapshot that parses correctly, even if it looks stale.
pub fn prune() -> Result<usize> {
    let mut removed = 0;
    for entry in list_all_raw()? {
        if entry.snapshot.is_err() {
            delete_raw_file(&entry.name)?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Lists all saved sessions with their disk usage, ordered from newest to oldest.
/// Corrupted files are skipped here — use `list_all_raw` or `state::all` to see them.
/// Kept as plain public API (used by tests, and by any consumer that only cares
/// about valid saved snapshots without paying for a live tmux query).
#[allow(dead_code)]
pub fn list() -> Result<Vec<SessionSummary>> {
    let mut out = Vec::new();
    for entry in list_all_raw()? {
        if let Ok(snapshot) = entry.snapshot {
            out.push(SessionSummary {
                pane_count: snapshot.pane_count(),
                window_count: snapshot.window_count(),
                name: snapshot.name,
                created_at: snapshot.created_at,
                size_bytes: entry.size_bytes,
            });
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

/// Formats bytes in a readable way (KB/MB), consistent with what is displayed in the picker.
pub fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    let b = bytes as f64;
    if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::WindowSnapshot;
    use std::sync::Mutex;

    // sessions_dir() reads $XDG_DATA_HOME / dirs::data_local_dir(), so tests that
    // touch it must not run concurrently with each other.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_storage<F: FnOnce()>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempdir();
        std::env::set_var("XDG_DATA_HOME", &tmp);
        f();
        std::env::remove_var("XDG_DATA_HOME");
        let _ = fs::remove_dir_all(&tmp);
    }

    fn tempdir() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "sess-storage-test-{}-{}",
            std::process::id(),
            fastrand_u64()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    // Tiny pseudo-random helper so parallel test runs in the same process don't
    // collide on the same temp directory name — avoids pulling in a `rand` dep.
    fn fastrand_u64() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn sample(name: &str) -> SessionSnapshot {
        SessionSnapshot::new(
            name.into(),
            vec![WindowSnapshot {
                index: 0,
                name: "w".into(),
                layout: String::new(),
                panes: vec![],
            }],
            Default::default(),
        )
    }

    #[test]
    fn save_then_load_round_trips() {
        with_temp_storage(|| {
            let snap = sample("roundtrip");
            save(&snap, false).unwrap();
            let loaded = load("roundtrip").unwrap();
            assert_eq!(loaded.name, "roundtrip");
        });
    }

    #[test]
    fn save_without_force_refuses_to_overwrite() {
        with_temp_storage(|| {
            save(&sample("dup"), false).unwrap();
            let result = save(&sample("dup"), false);
            assert!(result.is_err());
        });
    }

    #[test]
    fn rename_moves_the_file_and_updates_name_field() {
        with_temp_storage(|| {
            save(&sample("old-name"), false).unwrap();
            rename("old-name", "new-name").unwrap();

            assert!(!exists("old-name").unwrap());
            let loaded = load("new-name").unwrap();
            assert_eq!(loaded.name, "new-name");
        });
    }

    #[test]
    fn prune_removes_only_unparseable_files() {
        with_temp_storage(|| {
            save(&sample("valid"), false).unwrap();
            let dir = sessions_dir().unwrap();
            fs::write(dir.join("broken.json"), "{ not valid json").unwrap();

            let removed = prune().unwrap();

            assert_eq!(removed, 1);
            assert!(exists("valid").unwrap());
            assert!(!dir.join("broken.json").exists());
        });
    }

    #[test]
    fn list_all_raw_reports_both_valid_and_broken() {
        with_temp_storage(|| {
            save(&sample("ok"), false).unwrap();
            let dir = sessions_dir().unwrap();
            fs::write(dir.join("bad.json"), "not json at all").unwrap();

            let raw = list_all_raw().unwrap();
            let ok_entry = raw.iter().find(|e| e.name == "ok").unwrap();
            let bad_entry = raw.iter().find(|e| e.name == "bad").unwrap();

            assert!(ok_entry.snapshot.is_ok());
            assert!(bad_entry.snapshot.is_err());
        });
    }
}
