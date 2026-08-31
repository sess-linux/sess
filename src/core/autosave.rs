use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::{capture, storage};

fn autosave_dir() -> Result<PathBuf> {
    let dir = storage::sessions_dir()?.join("autosave");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn pid_file(name: &str) -> Result<PathBuf> {
    Ok(autosave_dir()?.join(format!("{name}.pid")))
}

fn log_file(name: &str) -> Result<PathBuf> {
    Ok(autosave_dir()?.join(format!("{name}.log")))
}

/// Best-effort liveness check: on Linux, a process's /proc/<pid> entry exists
/// for exactly as long as the process does.
pub fn is_running(name: &str) -> bool {
    let Ok(path) = pid_file(name) else {
        return false;
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(pid) = raw.trim().parse::<u32>() else {
        return false;
    };
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// Spawns a background process that periodically re-saves `name`'s current
/// tmux state under itself, every `interval_secs` seconds, until stopped with
/// `stop()` or until the tmux session it watches disappears (at which point it
/// exits on its own).
///
/// Known limitation: the child is not fully daemonized (no double-fork /
/// setsid), so it depends on the OS not sending it SIGHUP when the parent
/// shell exits. On most Linux setups a detached child survives its parent
/// exiting; a proper daemon would be more robust but adds real complexity for
/// a feature meant to stay simple — see the design notes.
pub fn start(name: &str, interval_secs: u64) -> Result<PathBuf> {
    if is_running(name) {
        anyhow::bail!("auto-save is already running for '{name}'");
    }

    let exe = std::env::current_exe().context("could not resolve the sess binary path")?;
    let log_path = log_file(name)?;
    let log = fs::File::create(&log_path)?;
    let log_err = log.try_clone()?;

    let child = Command::new(exe)
        .arg("__autosave-loop")
        .arg(name)
        .arg(interval_secs.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err))
        .spawn()
        .context("could not start the auto-save background process")?;

    fs::write(pid_file(name)?, child.id().to_string())?;
    Ok(log_path)
}

pub fn stop(name: &str) -> Result<()> {
    let path = pid_file(name)?;
    let Ok(raw) = fs::read_to_string(&path) else {
        anyhow::bail!("auto-save is not running for '{name}'");
    };

    if let Ok(pid) = raw.trim().parse::<u32>() {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
    let _ = fs::remove_file(&path);
    Ok(())
}

/// The loop body itself, run inside the detached child process spawned by
/// `start`. Not meant to be called directly by users — invoked internally via
/// the hidden `__autosave-loop` CLI command.
pub fn run_loop(name: &str, interval_secs: u64) -> Result<()> {
    let interval = std::time::Duration::from_secs(interval_secs.max(1));

    loop {
        std::thread::sleep(interval);

        if !capture::tmux_session_exists(name) {
            break;
        }

        if let Ok(mut snapshot) = capture::capture_session(name) {
            snapshot.name = name.to_string();
            // Carry over previously persisted env vars instead of dropping them —
            // auto-save should never silently downgrade a session that was
            // explicitly saved with `sess save` earlier.
            if let Ok(previous) = storage::load(name) {
                snapshot.env = previous.env;
            }
            let _ = storage::save(&snapshot, true);
        }
    }

    let _ = fs::remove_file(pid_file(name)?);
    Ok(())
}
