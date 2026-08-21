//! Handing an [`Alert`] to the desktop (§6 as amended, bl-e160): one child per
//! alert, and the argv it becomes.
//!
//! **Zero new dependencies** (AGENTS.md rule 6). A desktop notification on
//! Linux is a D-Bus call to `org.freedesktop.Notifications`, and every Rust
//! crate that makes one (`notify-rust` and its `zbus`/`dbus` stack) is a new
//! dependency yog is not allowed to take. The freedesktop spec's own reference
//! client is a binary every desktop already ships — libnotify's `notify-send` —
//! and yog is a program whose whole substrate is spawned binaries. So this is
//! the spawn discipline yog already has, pointed one process further out.
//!
//! **Not through [`Cli`](crate::cli_outbound::Cli), deliberately.** Every `Cli`
//! spawn folds yog's composed world onto the child (§16.2) so the substrate
//! nests. The notifier is not substrate — it is the operator's own desktop
//! session, reached through the session-bus address yog itself inherited — so it
//! takes the bare [`git_env::command`](crate::git_env::command) constructor: the
//! ambient environment minus git's leaked `GIT_DIR`/`GIT_INDEX_FILE`, which is
//! what every child gets and no more.
//!
//! **Synchronous here, off-thread at the seat.** This blocks on each child, and
//! the frame must never block (bl-ee0a), so the window's one call site runs it
//! on a thread of its own. Keeping the wait here rather than hiding a spawn
//! inside is what lets a test drive it deterministically.
//!
//! **Every failure is silent.** A desktop with no notifier, a refused bus, a
//! non-zero exit: none is an event to record. `ops.jsonl` is the log of what
//! yog *did to the world* (§4.2) and a notification changes nothing — it is
//! render output that happens to leave the window. A notifier that is absent
//! renders nothing, exactly as the strip renders nothing when nothing stirs.

use std::path::Path;
use std::process::Stdio;

use super::Alert;

/// The desktop notifier yog spends — libnotify's reference client, resolved on
/// `PATH` like every other binary yog names by word rather than by path.
pub const NOTIFIER: &str = "notify-send";

/// How yog identifies itself to the notification daemon, so an operator's
/// desktop can filter or theme yog's notifications as yog's.
const APP_NAME: &str = "yog";

/// The argv one alert becomes: `notify-send -a yog <summary> <body>`.
///
/// Summary and body ride last and unescaped because they are exactly the two
/// positional operands libnotify takes, and they are yog-composed sentences —
/// a workspace leaf (§3.1-validated), a §3.3 display name, and the fixed rule
/// wording ([`AttentionKind::says`](crate::attention::AttentionKind::says)).
/// Nothing here is a shell string: the child is spawned directly, so there is
/// no word splitting to defend against.
pub fn argv(alert: &Alert) -> Vec<String> {
    vec![
        "-a".to_string(),
        APP_NAME.to_string(),
        alert.summary.clone(),
        alert.body.clone(),
    ]
}

/// Announce each alert, blocking on each child. `notifier` is the program to
/// run — [`NOTIFIER`] in the window, a stub in a test.
///
/// stdio is bound to null in all three directions: a notifier's chatter is not
/// yog's to carry, and a child inheriting the frame's stdout would interleave
/// with nothing anyone reads.
pub fn deliver(notifier: &Path, alerts: &[Alert]) {
    for alert in alerts {
        drop(crate::git_env::status(
            crate::git_env::command(notifier)
                .args(argv(alert))
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null()),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    fn alert(summary: &str, body: &str) -> Alert {
        Alert {
            summary: summary.to_string(),
            body: body.to_string(),
        }
    }

    /// The argv is libnotify's own: yog names itself, then the two positional
    /// operands, in that order and nothing between them.
    #[test]
    fn one_alert_is_app_name_then_summary_then_body() {
        assert_eq!(
            argv(&alert("cobalt · ochre-tern", "came to rest — your turn")),
            vec![
                "-a".to_string(),
                "yog".to_string(),
                "cobalt · ochre-tern".to_string(),
                "came to rest — your turn".to_string(),
            ]
        );
    }

    /// Every alert reaches the desktop, in order — driven onto a stub notifier
    /// that records what it was handed, so the assertion is on the real spawn
    /// rather than on a mock of one.
    #[test]
    fn every_alert_reaches_the_notifier_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("said");
        let stub = dir.path().join("notify-send");
        std::fs::write(
            &stub,
            format!(
                "#!/bin/sh\nprintf '%s|%s\\n' \"$3\" \"$4\" >> {}\n",
                log.display()
            ),
        )
        .unwrap();
        let mut perms = std::fs::metadata(&stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&stub, perms).unwrap();

        deliver(
            &stub,
            &[
                alert("cobalt · one", "came to rest"),
                alert("slate · two", "has mail"),
            ],
        );
        assert_eq!(
            std::fs::read_to_string(&log).unwrap(),
            "cobalt · one|came to rest\nslate · two|has mail\n"
        );
    }

    /// A desktop with no notifier is silent, not a panic and not an error path:
    /// the whole feature degrades to the world before it existed.
    #[test]
    fn an_absent_notifier_says_nothing_and_raises_nothing() {
        deliver(
            Path::new("/nonexistent/yog-has-no-notifier"),
            &[alert("cobalt · one", "came to rest")],
        );
    }
}
