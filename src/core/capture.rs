use anyhow::{bail, Context, Result};
use std::process::Command;

use super::model::{PaneSnapshot, SessionSnapshot, WindowSnapshot};

/// Runs a tmux command and returns stdout as text, or returns an error with the tmux message.
fn tmux(args: &[&str]) -> Result<String> {
    let out = Command::new("tmux")
        .args(args)
        .output()
        .context("could not execute tmux — is it installed and in the PATH?")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("tmux {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/// Returns the name of the tmux session from which `sess` is running.
/// Fails with a clear message if it is not being executed inside tmux.
pub fn current_session_name() -> Result<String> {
    if std::env::var("TMUX").is_err() {
        bail!(
            "you are not inside a tmux session.\n\n\
             sess needs to run from inside tmux to know what to save.\n\
             If you want to start a new tracked session with sess, use:\n\n  sess start <name>"
        );
    }
    tmux(&["display-message", "-p", "#{session_name}"])
}

/// Checks whether a tmux session with that name is already running.
pub fn tmux_session_exists(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Lists the names of every currently running tmux session. Returns an empty
/// list (not an error) if tmux has no server running at all — that's a normal
/// state, not a failure.
pub fn list_live_sessions() -> Result<Vec<String>> {
    let out = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output();

    let out = match out {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };

    if !out.status.success() {
        // Most common cause: no tmux server running yet. Not an error for our purposes.
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

/// Captures the complete state (windows + panes) of a tmux session by name.
pub fn capture_session(session_name: &str) -> Result<SessionSnapshot> {
    // One line per window: index, name, layout.
    let windows_raw = tmux(&[
        "list-windows",
        "-t",
        session_name,
        "-F",
        "#{window_index}\t#{window_name}\t#{window_layout}",
    ])?;

    let mut windows = Vec::new();

    for line in windows_raw.lines() {
        let mut parts = line.splitn(3, '\t');
        let index: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let name = parts.next().unwrap_or("").to_string();
        let layout = parts.next().unwrap_or("").to_string();

        let target = format!("{session_name}:{index}");
        let panes_raw = tmux(&[
            "list-panes",
            "-t",
            &target,
            "-F",
            "#{pane_index}\t#{pane_current_path}\t#{pane_current_command}\t#{pane_pid}",
        ])?;

        let panes: Vec<PaneSnapshot> = panes_raw
            .lines()
            .map(|l| {
                let mut p = l.splitn(4, '\t');
                let index: u32 = p.next().unwrap_or("0").parse().unwrap_or(0);
                let cwd = p.next().unwrap_or("").to_string();
                let short_command = p.next().unwrap_or("").to_string();
                let pid: u32 = p.next().unwrap_or("0").parse().unwrap_or(0);

                // Idle shells and sess capturing itself mid-command both mean
                // "nothing meaningful to relaunch".
                let command = if is_bare_shell(&short_command) || is_sess_itself(&short_command) {
                    String::new()
                } else {
                    // pane_pid is the shell PID for the pane, not the foreground process.
                    // We search through descendants for the process whose name matches the
                    // one tmux already identified, then read its full command line from /proc.
                    full_cmdline_of_foreground(pid, &short_command).unwrap_or(short_command)
                };
                PaneSnapshot {
                    index,
                    cwd,
                    command,
                }
            })
            .collect();

        windows.push(WindowSnapshot {
            index,
            name,
            layout,
            panes,
        });
    }

    if windows.is_empty() {
        bail!("no windows were found in session '{session_name}'");
    }

    Ok(SessionSnapshot::new(
        session_name.to_string(),
        windows,
        Default::default(),
    ))
}

fn is_bare_shell(cmd: &str) -> bool {
    matches!(cmd, "bash" | "zsh" | "fish" | "sh" | "dash")
}

/// True when `short_command` is sess's own binary name. A pane running
/// `sess save` (or `sess open`, etc.) captures *itself* as the pane's
/// foreground process at the instant of capture — it hasn't returned to the
/// shell prompt yet. Relaunching that on restore would silently re-run a
/// `sess` command (most dangerously `sess save ... --force`, which can
/// resurrect an old snapshot), so it's treated the same as an idle shell:
/// nothing meaningful to relaunch.
fn is_sess_itself(short_command: &str) -> bool {
    let own_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
    match own_name {
        Some(name) => short_command == name,
        None => short_command == "sess",
    }
}

/// Given a pane shell PID, search among its descendants for the foreground process
/// (the one tmux already identified by short name) and return its full command line,
/// including arguments, read from /proc. This only works on Linux.
fn full_cmdline_of_foreground(shell_pid: u32, short_command: &str) -> Option<String> {
    let mut frontier = vec![shell_pid];
    let mut depth = 0;

    // BFS is bounded: the foreground process is usually a direct child of the shell,
    // but it may be one level deeper if invoked via `sh -c ...`.
    while !frontier.is_empty() && depth < 3 {
        let mut next_frontier = Vec::new();
        for pid in &frontier {
            for child in children_of(*pid) {
                if comm_matches(child, short_command) {
                    if let Some(cmd) = cmdline_of(child) {
                        return Some(cmd);
                    }
                }
                next_frontier.push(child);
            }
        }
        frontier = next_frontier;
        depth += 1;
    }
    None
}

fn children_of(pid: u32) -> Vec<u32> {
    let path = format!("/proc/{pid}/task/{pid}/children");
    std::fs::read_to_string(path)
        .ok()
        .map(|s| {
            s.split_whitespace()
                .filter_map(|p| p.parse().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn comm_matches(pid: u32, short_command: &str) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|c| c.trim() == short_command)
        .unwrap_or(false)
}

fn cmdline_of(pid: u32) -> Option<String> {
    let raw = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    if raw.is_empty() {
        return None;
    }
    let cmd = raw
        .split(|&b| b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).to_string())
        .collect::<Vec<_>>()
        .join(" ");
    if cmd.is_empty() {
        None
    } else {
        Some(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_bare_shell_matches_known_shells_only() {
        assert!(is_bare_shell("bash"));
        assert!(is_bare_shell("zsh"));
        assert!(is_bare_shell("fish"));
        assert!(!is_bare_shell("vim"));
        assert!(!is_bare_shell("sess"));
    }

    #[test]
    fn is_sess_itself_matches_the_running_binarys_name() {
        // In `cargo test`, the running binary is the test harness, not `sess` —
        // this exercises the fallback branch, which is what a real `sess`
        // build would hit via std::env::current_exe() succeeding with "sess".
        assert!(!is_sess_itself("bash"));
        assert!(!is_sess_itself("vim"));
    }
}
