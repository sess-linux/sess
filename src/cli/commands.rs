use anyhow::{Context, Result};
use std::io::{self, Write};
use std::process::Command;

use crate::core::{capture, restore, storage};

fn run_tmux(args: &[&str]) -> Result<()> {
    let out = Command::new("tmux")
        .args(args)
        .output()
        .context("could not execute tmux")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("tmux {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(())
}

pub fn start(name: String) -> Result<()> {
    if capture::tmux_session_exists(&name) {
        println!("'{name}' is already running, attaching...");
        return restore::attach(&name);
    }
    restore::start_new(&name)?;
    restore::attach(&name)
}

pub fn save(name: Option<String>, force: bool) -> Result<()> {
    let session_name = capture::current_session_name()?;
    let saved_as = name.unwrap_or_else(|| session_name.clone());

    let already_exists = storage::exists(&saved_as)?;
    if already_exists && !force {
        print!(
            "a saved session named '{saved_as}' already exists. Overwrite? [y/N] "
        );
        io::stdout().flush().ok();
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            println!("cancelled.");
            return Ok(());
        }
    }

    let mut snapshot = capture::capture_session(&session_name)?;
    snapshot.name = saved_as.clone();
    storage::save(&snapshot, true)?;

    let pane_count: usize = snapshot.windows.iter().map(|w| w.panes.len()).sum();
    println!("session saved as '{saved_as}' ({pane_count} panes).");
    Ok(())
}

pub fn list(json: bool) -> Result<()> {
    let sessions = storage::list()?;
    if json {
        let json = serde_json::to_string_pretty(&sessions)?;
        println!("{json}");
        return Ok(());
    }

    if sessions.is_empty() {
        println!("no saved sessions yet. Use `sess save <name>` to create one.");
        return Ok(());
    }

    println!("{:<20} {:>8} {:>10}  {}", "NAME", "PANES", "SIZE", "SAVED");
    for s in &sessions {
        println!(
            "{:<20} {:>8} {:>10}  {}",
            s.name,
            s.pane_count,
            storage::human_size(s.size_bytes),
            s.created_at.format("%Y-%m-%d %H:%M")
        );
    }
    Ok(())
}

pub fn open(name: String, force: bool) -> Result<()> {
    if capture::tmux_session_exists(&name) && !force {
        println!("'{name}' is already running, attaching...");
        return restore::attach(&name);
    }

    let snapshot = storage::load(&name)?;
    if capture::tmux_session_exists(&name) && force {
        run_tmux(&["kill-session", "-t", &name])?;
    }

    restore::restore(&snapshot)?;
    println!("session '{name}' restored.");
    restore::attach(&name)
}

pub fn attach(name: String) -> Result<()> {
    if !capture::tmux_session_exists(&name) {
        anyhow::bail!("tmux session '{name}' is not running; use `sess open {name}` to restore it from a saved snapshot");
    }
    restore::attach(&name)
}

pub fn delete(name: String) -> Result<()> {
    if capture::tmux_session_exists(&name) {
        run_tmux(&["kill-session", "-t", &name])?;
        println!("tmux session '{name}' killed.");
    }

    if storage::exists(&name)? {
        storage::delete(&name)?;
        println!("saved session '{name}' deleted.");
    }

    Ok(())
}

pub fn rename(from: String, to: String) -> Result<()> {
    if from == to {
        anyhow::bail!("source and destination names are the same");
    }
    storage::rename(&from, &to)?;
    println!("session '{from}' renamed to '{to}'.");
    Ok(())
}

pub fn duplicate(from: String, to: String) -> Result<()> {
    if from == to {
        anyhow::bail!("source and destination names are the same");
    }

    let snapshot = storage::load(&from)?;
    let mut snapshot = snapshot;
    snapshot.name = to.clone();
    storage::save(&snapshot, false)?;
    println!("session '{from}' duplicated to '{to}'.");
    Ok(())
}

pub fn status() -> Result<()> {
    let saved = storage::list()?;
    let live = tmux_sessions()?;

    println!("saved sessions: {}", saved.len());
    println!("live tmux sessions: {}", live.len());

    if !saved.is_empty() {
        println!("saved names:");
        for s in &saved {
            println!("  - {}", s.name);
        }
    }

    if !live.is_empty() {
        println!("live names:");
        for name in &live {
            println!("  - {}", name);
        }
    }

    Ok(())
}

fn tmux_sessions() -> Result<Vec<String>> {
    let out = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .context("could not list tmux sessions")?;

    if !out.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

pub fn prune() -> Result<()> {
    let removed = storage::prune()?;
    println!("removed {removed} invalid saved session file(s).");
    Ok(())
}

pub fn kill_session(name: String) -> Result<()> {
    run_tmux(&["kill-session", "-t", &name])?;

    if storage::exists(&name)? {
        storage::delete(&name)?;
        println!("saved session '{name}' deleted.");
    }

    println!("session '{name}' killed.");
    Ok(())
}

pub fn kill_window(target: String) -> Result<()> {
    run_tmux(&["kill-window", "-t", &target])?;
    println!("window '{target}' killed.");
    Ok(())
}

pub fn kill_server() -> Result<()> {
    run_tmux(&["kill-server"])?;
    println!("tmux server killed.");
    Ok(())
}
