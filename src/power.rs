//! Reading and flipping `pmset disablesleep`, plus the optional sudoers rule
//! that lets the switch flip it without a password prompt.

use std::path::Path;
use std::process::{Command, Stdio};

const PMSET: &str = "/usr/bin/pmset";
const SUDO: &str = "/usr/bin/sudo";
pub const SUDOERS_FILE: &str = "/etc/sudoers.d/sleep-switch";

/// `Some(true)` when sleep is disabled, `None` if pmset could not be read.
pub fn sleep_disabled() -> Option<bool> {
    let out = Command::new(PMSET).arg("-g").output().ok()?;
    String::from_utf8_lossy(&out.stdout).lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        match (parts.next(), parts.next()) {
            (Some("SleepDisabled"), Some(v)) => Some(v == "1"),
            _ => None,
        }
    })
}

/// Flip the setting. Tries the password-free sudo rule first, then falls
/// back to the standard macOS administrator password dialog.
pub fn set_sleep_disabled(disabled: bool) -> Result<(), String> {
    let v = if disabled { "1" } else { "0" };
    let quiet = Command::new(SUDO)
        .args(["-n", PMSET, "-a", "disablesleep", v])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if matches!(quiet, Ok(s) if s.success()) {
        return Ok(());
    }
    run_as_admin(&format!("{PMSET} -a disablesleep {v}"))
}

pub fn password_free_installed() -> bool {
    Path::new(SUDOERS_FILE).exists()
}

/// Install a sudoers rule allowing exactly `pmset -a disablesleep 0` and
/// `pmset -a disablesleep 1` for the current user, nothing else. The rule is
/// checked with `visudo -c` before it goes in, so a bad file can never lock
/// sudo out.
pub fn install_password_free() -> Result<(), String> {
    let user = std::env::var("USER").map_err(|_| "USER is not set".to_string())?;
    if user.is_empty() || !user.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)) {
        return Err(format!("unexpected user name {user:?}"));
    }
    let rule = format!(
        "# Installed by Sleep Switch. Delete this file to remove it.\n\
         {user} ALL=(root) NOPASSWD: {PMSET} -a disablesleep 0, {PMSET} -a disablesleep 1\n"
    );
    let tmp = std::env::temp_dir().join(format!("sleep-switch-{}.sudoers", std::process::id()));
    std::fs::write(&tmp, rule).map_err(|e| format!("writing {}: {e}", tmp.display()))?;
    let tmp_s = tmp.display().to_string();
    if tmp_s.contains(|c: char| c == '\'' || c == '"' || c == '\\' || c.is_whitespace()) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("temp path needs quoting: {tmp_s}"));
    }
    let result = run_as_admin(&format!(
        "/usr/sbin/visudo -cf {tmp_s} && /usr/bin/install -m 0440 -o root -g wheel {tmp_s} {SUDOERS_FILE}"
    ));
    let _ = std::fs::remove_file(&tmp);
    result
}

pub fn remove_password_free() -> Result<(), String> {
    run_as_admin(&format!("/bin/rm -f {SUDOERS_FILE}"))
}

/// Run a shell command as root via the macOS password dialog. `cmd` must not
/// contain double quotes or backslashes; every caller builds it from fixed
/// paths.
fn run_as_admin(cmd: &str) -> Result<(), String> {
    debug_assert!(!cmd.contains('"') && !cmd.contains('\\'));
    let script = format!("do shell script \"{cmd}\" with administrator privileges");
    let out = Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("osascript: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    if err.contains("-128") {
        Err("cancelled".to_string())
    } else {
        Err(err.trim().to_string())
    }
}
