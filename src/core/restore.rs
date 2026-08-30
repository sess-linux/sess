use anyhow::{Context, Result};
use std::process::Command;

use super::capture::tmux_session_exists;
use super::model::SessionSnapshot;

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

/// Recreates the tmux session from the saved snapshot: windows, panes,
/// exact geometry (via the layout string), and relaunches the commands that were running.
///
/// If a live tmux session with that name already exists, it is left alone — the caller
/// is expected to simply attach instead (see cli::open).
pub fn restore(snapshot: &SessionSnapshot) -> Result<()> {
    if tmux_session_exists(&snapshot.name) {
        anyhow::bail!(
            "a live tmux session named '{}' already exists — use `sess open {}` to attach instead of restoring.",
            snapshot.name,
            snapshot.name
        );
    }

    let Some(first_window) = snapshot.windows.first() else {
        anyhow::bail!("the snapshot has no saved windows");
    };
    let Some(first_pane) = first_window.panes.first() else {
        anyhow::bail!("the first window in the snapshot has no saved panes");
    };

    // 1. Create the session with the first window, located in the cwd of the first pane.
    run_tmux(&[
        "new-session",
        "-d",
        "-s",
        &snapshot.name,
        "-c",
        &first_pane.cwd,
        "-n",
        &first_window.name,
    ])?;

    for (i, window) in snapshot.windows.iter().enumerate() {
        let window_target = format!("{}:{}", snapshot.name, window.index);

        if i > 0 {
            // Remaining windows: create them explicitly with the original index and name.
            run_tmux(&[
                "new-window",
                "-t",
                &format!("{}:", snapshot.name),
                "-n",
                &window.name,
                "-c",
                &window.panes.first().map(|p| p.cwd.as_str()).unwrap_or("~"),
            ])?;
        }

        // 2. Split windows: one for each extra pane, all starting from pane 0.
        for pane in window.panes.iter().skip(1) {
            run_tmux(&[
                "split-window",
                "-t",
                &window_target,
                "-c",
                &pane.cwd,
            ])?;
        }

        // 3. Apply the exact saved geometry (avoids the splits being evenly distributed).
        if !window.layout.is_empty() {
            run_tmux(&["select-layout", "-t", &window_target, &window.layout])?;
        }

        // 4. Relaunch the commands that were running, pane by pane.
        for pane in &window.panes {
            if pane.command.is_empty() {
                continue;
            }
            let pane_target = format!("{}.{}", window_target, pane.index);
            // "--" prevents an argument starting with "-" from being interpreted as a flag.
            run_tmux(&["send-keys", "-t", &pane_target, "--", &pane.command, "Enter"])?;
        }
    }

    // Leave the first window selected, as it was originally.
    run_tmux(&["select-window", "-t", &format!("{}:{}", snapshot.name, first_window.index)])?;

    Ok(())
}

/// Creates a new empty tmux session with the given name (for `sess start`).
pub fn start_new(name: &str) -> Result<()> {
    run_tmux(&["new-session", "-d", "-s", name])
}

/// Attaches to (or switches to) an existing tmux session.
pub fn attach(name: &str) -> Result<()> {
    let inside_tmux = std::env::var("TMUX").is_ok();
    let flag = if inside_tmux { "switch-client" } else { "attach-session" };
    run_tmux(&[flag, "-t", name])
}
