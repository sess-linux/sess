use std::collections::BTreeMap;

/// Substrings (case-insensitive) that flag a variable name as likely holding a
/// secret. Matched against the allow-list, not against the whole environment —
/// sess never scans the environment on its own.
const SECRET_MARKERS: &[&str] = &["KEY", "TOKEN", "SECRET", "PASSWORD", "AUTH", "CREDENTIAL"];

pub fn looks_like_secret(name: &str) -> bool {
    let upper = name.to_uppercase();
    SECRET_MARKERS.iter().any(|marker| upper.contains(marker))
}

/// Reads the current process environment and returns only the variables that
/// are both present and explicitly named in `allowlist`. This is the only way
/// environment variables ever end up in a snapshot — there is no "persist
/// everything" mode.
pub fn capture_allowed(allowlist: &[String]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for name in allowlist {
        if let Ok(value) = std::env::var(name) {
            out.insert(name.clone(), value);
        }
    }
    out
}

/// Quotes a value for safe use inside a single-quoted POSIX shell string.
pub fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_common_secret_names() {
        assert!(looks_like_secret("API_KEY"));
        assert!(looks_like_secret("AWS_SECRET_ACCESS_KEY"));
        assert!(looks_like_secret("GITHUB_TOKEN"));
        assert!(looks_like_secret("DB_PASSWORD"));
        assert!(looks_like_secret("BASIC_AUTH"));
        assert!(looks_like_secret("SSH_CREDENTIAL"));
    }

    #[test]
    fn does_not_flag_ordinary_names() {
        assert!(!looks_like_secret("NODE_ENV"));
        assert!(!looks_like_secret("EDITOR"));
        assert!(!looks_like_secret("PROJECT_ENV"));
    }

    #[test]
    fn only_captures_allow_listed_present_vars() {
        std::env::set_var("SESS_TEST_ENV_ONE", "value-one");
        std::env::remove_var("SESS_TEST_ENV_MISSING");

        let allowlist = vec![
            "SESS_TEST_ENV_ONE".to_string(),
            "SESS_TEST_ENV_MISSING".to_string(),
        ];
        let captured = capture_allowed(&allowlist);

        assert_eq!(
            captured.get("SESS_TEST_ENV_ONE"),
            Some(&"value-one".to_string())
        );
        assert!(!captured.contains_key("SESS_TEST_ENV_MISSING"));
        std::env::remove_var("SESS_TEST_ENV_ONE");
    }

    #[test]
    fn quotes_single_quotes_safely() {
        assert_eq!(shell_single_quote("plain"), "'plain'");
        assert_eq!(shell_single_quote("it's"), r"'it'\''s'");
    }
}
