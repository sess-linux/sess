use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

use super::model::{SessionSnapshot, SessionSummary};

/// ~/.local/share/sess/
fn sessions_dir() -> Result<PathBuf> {
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

pub fn prune() -> Result<usize> {
    let dir = sessions_dir()?;
    let mut removed = 0;

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(_) => {
                fs::remove_file(&path)?;
                removed += 1;
                continue;
            }
        };

        if serde_json::from_str::<SessionSnapshot>(&raw).is_err() {
            fs::remove_file(&path)?;
            removed += 1;
        }
    }

    Ok(removed)
}

/// Lists all saved sessions with their disk usage, ordered from newest to oldest.
pub fn list() -> Result<Vec<SessionSummary>> {
    let dir = sessions_dir()?;
    let mut out = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        let snapshot: SessionSnapshot = match serde_json::from_str(&raw) {
            Ok(s) => s,
            Err(_) => continue, // ignore corrupt files instead of breaking the entire listing
        };
        let size_bytes = entry.metadata()?.len();
        let pane_count = snapshot.windows.iter().map(|w| w.panes.len()).sum();

        out.push(SessionSummary {
            name: snapshot.name,
            created_at: snapshot.created_at,
            pane_count,
            size_bytes,
        });
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
