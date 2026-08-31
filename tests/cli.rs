//! Integration tests that exercise the built `sess` binary directly, as a
//! separate suite from the unit tests in `src/`, per the project's testing
//! guidelines. These deliberately avoid requiring a real tmux session to be
//! running — they only check commands that must behave safely with or
//! without one.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_sess")
}

#[test]
fn help_lists_the_v0_3_commands() {
    let output = Command::new(bin())
        .arg("--help")
        .output()
        .expect("failed to run sess --help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    for expected in ["switch", "close", "doctor", "auto-save", "status", "prune"] {
        assert!(
            stdout.contains(expected),
            "expected --help to mention '{expected}', got:\n{stdout}"
        );
    }
}

#[test]
fn doctor_runs_without_panicking_regardless_of_environment() {
    // Doctor must degrade gracefully (Fail/Warn results) rather than crash,
    // whether or not tmux happens to be installed in the test environment.
    let output = Command::new(bin())
        .arg("doctor")
        .output()
        .expect("failed to run sess doctor");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tmux installed"));
}

#[test]
fn save_outside_tmux_fails_with_a_clear_message_not_a_panic() {
    let output = Command::new(bin())
        .arg("save")
        .env_remove("TMUX")
        .output()
        .expect("failed to run sess save");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tmux session"));
}

#[test]
fn list_json_is_valid_json_even_when_empty() {
    let tmp = std::env::temp_dir().join(format!("sess-cli-test-list-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let output = Command::new(bin())
        .args(["list", "--json"])
        .env("XDG_DATA_HOME", &tmp)
        .output()
        .expect("failed to run sess list --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");

    let _ = std::fs::remove_dir_all(&tmp);
}
