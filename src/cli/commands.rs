use anyhow::{Context, Result};
use std::io::{self, Write};
use std::process::Command;

use crate::core::state::StateKind;
use crate::core::{autosave, capture, config, env, restore, state, storage};

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

pub fn start(name: String, auto_save: bool) -> Result<()> {
    if capture::tmux_session_exists(&name) {
        println!("'{name}' is already running, attaching...");
        return restore::attach(&name);
    }
    restore::start_new(&name)?;

    if auto_save {
        let cfg = config::load();
        let interval = cfg.autosave.interval;
        match autosave::start(&name, interval) {
            Ok(log) => println!(
                "auto-save enabled for '{name}' (every {interval}s, log: {})",
                log.display()
            ),
            Err(e) => eprintln!("warning: could not start auto-save: {e}"),
        }
    }

    restore::attach(&name)
}

pub fn save(name: Option<String>, force: bool) -> Result<()> {
    let session_name = capture::current_session_name()?;
    let saved_as = name.unwrap_or_else(|| session_name.clone());

    let already_exists = storage::exists(&saved_as)?;
    if already_exists && !force {
        print!("a saved session named '{saved_as}' already exists. Overwrite? [y/N] ");
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

    let cfg = config::load();
    if !cfg.environment.persist.is_empty() {
        let captured = env::capture_allowed(&cfg.environment.persist);
        for name in captured.keys() {
            if env::looks_like_secret(name) {
                println!(
                    "warning: '{name}' looks like it might hold a secret — persisting it anyway because it's explicitly listed in your config."
                );
            }
        }
        snapshot.env = captured;
    }

    storage::save(&snapshot, true)?;

    let pane_count = snapshot.pane_count();
    if snapshot.env.is_empty() {
        println!("session saved as '{saved_as}' ({pane_count} panes).");
    } else {
        println!(
            "session saved as '{saved_as}' ({pane_count} panes, {} env var(s) persisted).",
            snapshot.env.len()
        );
    }
    Ok(())
}

pub fn list(json: bool) -> Result<()> {
    let states = state::all()?;

    if json {
        let json = serde_json::to_string_pretty(&states)?;
        println!("{json}");
        return Ok(());
    }

    if states.is_empty() {
        println!("no sessions yet. Use `sess start <name>` or `sess save <name>` to create one.");
        return Ok(());
    }

    println!(
        "{:<20} {:<8} {:>3} {:>3} {:>10}  SAVED",
        "NAME", "STATE", "WIN", "PANE", "SIZE"
    );
    for s in &states {
        let saved = s
            .saved_at
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());
        let size = s
            .size_bytes
            .map(storage::human_size)
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<20} {:<8} {:>3} {:>3} {:>10}  {}",
            s.name,
            s.kind.to_string(),
            s.window_count
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_string()),
            s.pane_count
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_string()),
            size,
            saved
        );
        if let Some(detail) = &s.detail {
            println!("  {}", detail);
        }
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
    if !snapshot.env.is_empty() {
        println!(
            "restoring {} persisted env var(s): {}",
            snapshot.env.len(),
            snapshot.env.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    println!("session '{name}' restored.");
    restore::attach(&name)
}

pub fn switch() -> Result<()> {
    crate::run_picker()
}

pub fn attach(name: String) -> Result<()> {
    if !capture::tmux_session_exists(&name) {
        anyhow::bail!("tmux session '{name}' is not running; use `sess open {name}` to restore it from a saved snapshot");
    }
    restore::attach(&name)
}

/// Closes a running session (kills the live tmux session) without touching
/// its saved snapshot. The counterpart of `open`: `close` then `open` should
/// round-trip back to the same state.
pub fn close(name: String) -> Result<()> {
    if !capture::tmux_session_exists(&name) {
        let hint = match state::of(&name)? {
            Some(s) if s.kind == StateKind::Saved => " (it has a saved snapshot but isn't running)",
            Some(s) if s.kind == StateKind::Stale => {
                " (it has a stale saved snapshot but isn't running)"
            }
            Some(s) if s.kind == StateKind::Broken => {
                " (its saved snapshot is broken and it isn't running)"
            }
            _ => "",
        };
        anyhow::bail!("'{name}' is not currently running{hint}");
    }
    run_tmux(&["kill-session", "-t", &name])?;
    println!("session '{name}' closed. Its saved snapshot (if any) was not touched.");
    Ok(())
}

/// Deletes a saved session's snapshot only. Does not touch a live tmux
/// session with the same name — use `close` for that, or `kill-session` to do
/// both at once.
pub fn delete(name: String) -> Result<()> {
    if !storage::exists(&name)? {
        anyhow::bail!("no saved session named '{name}' exists");
    }
    storage::delete(&name)?;
    println!("saved session '{name}' deleted.");

    if capture::tmux_session_exists(&name) {
        println!("note: '{name}' is still running as a live tmux session — use `sess close {name}` to stop it too.");
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

    let mut snapshot = storage::load(&from)?;
    snapshot.name = to.clone();
    storage::save(&snapshot, false)?;
    println!("session '{from}' duplicated to '{to}'.");
    Ok(())
}

pub fn status() -> Result<()> {
    let states = state::all()?;

    let running = states
        .iter()
        .filter(|s| s.kind == StateKind::Running)
        .count();
    let saved = states.iter().filter(|s| s.kind == StateKind::Saved).count();
    let stale = states.iter().filter(|s| s.kind == StateKind::Stale).count();
    let broken = states
        .iter()
        .filter(|s| s.kind == StateKind::Broken)
        .count();

    println!(
        "{running} running, {saved} saved, {stale} stale, {broken} broken ({} total)",
        states.len()
    );

    for s in &states {
        let extra = s
            .detail
            .as_deref()
            .map(|d| format!(" — {d}"))
            .unwrap_or_default();
        println!("  {:<8} {}{}", s.kind.to_string(), s.name, extra);
    }

    Ok(())
}

pub fn doctor(fix: bool) -> Result<()> {
    use crate::core::doctor::{run_checks, CheckStatus};

    let results = run_checks();
    for r in &results {
        let icon = match r.status {
            CheckStatus::Ok => "\u{2713}",
            CheckStatus::Warn => "\u{26a0}",
            CheckStatus::Fail => "\u{2717}",
        };
        println!("{icon} {}", r.label);
        if let Some(detail) = &r.detail {
            println!("    {detail}");
        }
    }

    if fix {
        println!();
        println!("applying safe fixes...");
        for r in crate::core::doctor::fix() {
            println!("  - {}", r.label);
        }
    }

    let code = crate::core::doctor::exit_code(&results);
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

pub fn auto_save(name: String, stop: bool, interval: Option<u64>) -> Result<()> {
    if stop {
        autosave::stop(&name)?;
        println!("auto-save stopped for '{name}'.");
        return Ok(());
    }

    if !capture::tmux_session_exists(&name) {
        anyhow::bail!(
            "'{name}' is not currently running — start it first with `sess start {name}`"
        );
    }

    let cfg = config::load();
    let interval = interval.unwrap_or(cfg.autosave.interval);
    let log = autosave::start(&name, interval)?;
    println!(
        "auto-save started for '{name}' (every {interval}s). Log: {}",
        log.display()
    );
    Ok(())
}

pub fn auto_save_loop(name: String, interval: u64) -> Result<()> {
    autosave::run_loop(&name, interval)
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
