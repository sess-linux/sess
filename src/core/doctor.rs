use std::process::Command;

use super::{state, storage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub status: CheckStatus,
    pub label: String,
    pub detail: Option<String>,
}

/// Runs every health check and returns the results in a fixed, predictable
/// order. Never panics — each check catches its own failures and reports them
/// as a Fail/Warn result instead.
pub fn run_checks() -> Vec<CheckResult> {
    vec![
        check_tmux_installed(),
        check_tmux_version(),
        check_storage_dir(),
        check_storage_writable(),
        check_session_data(),
        check_terminal_environment(),
    ]
}

/// Overall process exit code for `sess doctor`: 1 if any check failed, 0 otherwise.
/// Warnings alone do not fail the command — they're informational.
pub fn exit_code(results: &[CheckResult]) -> i32 {
    if results.iter().any(|r| r.status == CheckStatus::Fail) {
        1
    } else {
        0
    }
}

fn check_tmux_installed() -> CheckResult {
    match Command::new("tmux").arg("-V").output() {
        Ok(out) if out.status.success() => CheckResult {
            status: CheckStatus::Ok,
            label: "tmux installed".into(),
            detail: None,
        },
        _ => CheckResult {
            status: CheckStatus::Fail,
            label: "tmux installed".into(),
            detail: Some("tmux was not found on your PATH — install it with your package manager (e.g. `apt install tmux`).".into()),
        },
    }
}

fn check_tmux_version() -> CheckResult {
    match Command::new("tmux").arg("-V").output() {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            CheckResult {
                status: CheckStatus::Ok,
                label: format!("tmux version ({version})"),
                detail: None,
            }
        }
        _ => CheckResult {
            status: CheckStatus::Warn,
            label: "tmux version".into(),
            detail: Some("could not determine the tmux version.".into()),
        },
    }
}

fn check_storage_dir() -> CheckResult {
    match storage::sessions_dir() {
        Ok(dir) => CheckResult {
            status: CheckStatus::Ok,
            label: format!("storage directory ({})", dir.display()),
            detail: None,
        },
        Err(e) => CheckResult {
            status: CheckStatus::Fail,
            label: "storage directory".into(),
            detail: Some(format!("could not create or access it: {e}")),
        },
    }
}

fn check_storage_writable() -> CheckResult {
    let Ok(dir) = storage::sessions_dir() else {
        return CheckResult {
            status: CheckStatus::Fail,
            label: "storage writable".into(),
            detail: Some("skipped — storage directory itself is not accessible.".into()),
        };
    };

    let probe = dir.join(".sess-doctor-write-check");
    match std::fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            CheckResult {
                status: CheckStatus::Ok,
                label: "storage writable".into(),
                detail: None,
            }
        }
        Err(e) => CheckResult {
            status: CheckStatus::Fail,
            label: "storage writable".into(),
            detail: Some(format!(
                "could not write to the storage directory: {e}. Check its permissions."
            )),
        },
    }
}

fn check_session_data() -> CheckResult {
    match state::all() {
        Ok(states) => {
            let broken = states
                .iter()
                .filter(|s| s.kind == state::StateKind::Broken)
                .count();
            let stale = states
                .iter()
                .filter(|s| s.kind == state::StateKind::Stale)
                .count();

            if broken > 0 {
                CheckResult {
                    status: CheckStatus::Fail,
                    label: "session data valid".into(),
                    detail: Some(format!(
                        "{broken} saved session file(s) could not be read. Run `sess prune` to remove them, or `sess doctor --fix`."
                    )),
                }
            } else if stale > 0 {
                CheckResult {
                    status: CheckStatus::Warn,
                    label: "session data valid".into(),
                    detail: Some(format!(
                        "{stale} saved session(s) reference directories that no longer exist."
                    )),
                }
            } else {
                CheckResult {
                    status: CheckStatus::Ok,
                    label: "session data valid".into(),
                    detail: None,
                }
            }
        }
        Err(e) => CheckResult {
            status: CheckStatus::Fail,
            label: "session data valid".into(),
            detail: Some(format!("could not inspect saved sessions: {e}")),
        },
    }
}

fn check_terminal_environment() -> CheckResult {
    if std::env::var("TERM").is_err() {
        return CheckResult {
            status: CheckStatus::Warn,
            label: "terminal environment".into(),
            detail: Some("$TERM is not set — some tmux features may not behave correctly.".into()),
        };
    }
    CheckResult {
        status: CheckStatus::Ok,
        label: "terminal environment".into(),
        detail: None,
    }
}

/// Applies only safe, non-destructive fixes: currently, just (re)creating the
/// storage directory if it's missing. Never deletes data — that stays a
/// deliberate, explicit action (`sess prune`, `sess delete`).
pub fn fix() -> Vec<CheckResult> {
    let mut applied = Vec::new();

    if storage::sessions_dir().is_ok() {
        applied.push(CheckResult {
            status: CheckStatus::Ok,
            label: "storage directory".into(),
            detail: Some("already present (created if it was missing).".into()),
        });
    }

    applied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_is_zero_without_failures() {
        let results = vec![
            CheckResult {
                status: CheckStatus::Ok,
                label: "a".into(),
                detail: None,
            },
            CheckResult {
                status: CheckStatus::Warn,
                label: "b".into(),
                detail: None,
            },
        ];
        assert_eq!(exit_code(&results), 0);
    }

    #[test]
    fn exit_code_is_nonzero_with_any_failure() {
        let results = vec![
            CheckResult {
                status: CheckStatus::Ok,
                label: "a".into(),
                detail: None,
            },
            CheckResult {
                status: CheckStatus::Fail,
                label: "b".into(),
                detail: None,
            },
        ];
        assert_eq!(exit_code(&results), 1);
    }
}
