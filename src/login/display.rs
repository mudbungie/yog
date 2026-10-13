//! **The display a sign-in's browser opens on** (bl-bd71).
//!
//! `bz --login` finishes a browser flow by launching one (`xdg-open`), which
//! needs a display in its environment. The engine runs as a user service
//! that starts at `default.target`, before the graphical session has exported
//! `DISPLAY` and `WAYLAND_DISPLAY` into the user manager — so the service,
//! and every child it seats, has neither, and the launch dies with *no
//! DISPLAY environment variable specified* into the login tail the seat
//! paints. The service does carry `XDG_RUNTIME_DIR`, and a session that has
//! a display has left a socket: `$XDG_RUNTIME_DIR/wayland-<n>` and
//! `/tmp/.X11-unix/X<n>`. So the child is handed the display the sockets
//! name, where the engine's own environment names none.
//!
//! **A box with neither socket is handed nothing**, which is the headless
//! engine's case and is not an error: the browser flow fails there as it
//! always did, and the device-code flow is the one that fits. Ordering the
//! unit after the graphical session would have been the other fix, and it is
//! the wrong one: a headless box has no such target to order after.

use std::path::Path;

use crate::xdg::Env;

/// Where X11 leaves its sockets.
pub(crate) const X11_SOCKETS: &str = "/tmp/.X11-unix";

/// **The display pairs the sign-in child is handed**: none where `env`
/// already names a display, else what the sockets under `runtime` and `x11`
/// name — the lowest-numbered of each.
pub(crate) fn pairs(env: &Env, runtime: Option<&Path>, x11: &Path) -> Vec<(String, String)> {
    if env.var("DISPLAY").is_some() || env.var("WAYLAND_DISPLAY").is_some() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if let Some(name) = runtime.and_then(|dir| first(dir, "wayland-")) {
        out.push(("WAYLAND_DISPLAY".to_owned(), name));
    }
    if let Some(name) = first(x11, "X") {
        out.push((
            "DISPLAY".to_owned(),
            format!(":{}", name.trim_start_matches('X')),
        ));
    }
    out
}

/// The lowest-numbered entry in `dir` whose name is `prefix` and digits.
fn first(dir: &Path, prefix: &str) -> Option<String> {
    let mut names: Vec<(u32, String)> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let n: u32 = name.strip_prefix(prefix)?.parse().ok()?;
            Some((n, name))
        })
        .collect();
    names.sort();
    names.into_iter().next().map(|(_, name)| name)
}

/// [`pairs`] over the box this engine runs on: the runtime dir the
/// environment names, and X11's own socket directory.
pub(crate) fn discover(env: &Env) -> Vec<(String, String)> {
    let runtime = env.var("XDG_RUNTIME_DIR");
    pairs(
        env,
        runtime.as_deref().map(Path::new),
        Path::new(X11_SOCKETS),
    )
}

#[cfg(test)]
mod tests {
    use super::{discover, pairs};
    use crate::xdg::Env;
    use std::path::Path;

    fn sockets(dir: &Path, names: &[&str]) {
        for name in names {
            std::fs::write(dir.join(name), b"").expect("a socket stand-in");
        }
    }

    /// **No display named, sockets present: both pairs, lowest-numbered.**
    #[test]
    fn the_sockets_name_the_display_where_the_environment_does_not() {
        let tmp = tempfile::tempdir().expect("a tempdir");
        let (runtime, x11) = (tmp.path().join("run"), tmp.path().join("x11"));
        std::fs::create_dir_all(&runtime).expect("run");
        std::fs::create_dir_all(&x11).expect("x11");
        sockets(
            &runtime,
            &["wayland-1", "wayland-0.lock", "wayland-0", "bus"],
        );
        sockets(&x11, &["X1", "X0"]);
        let env = Env::from_pairs([("XDG_RUNTIME_DIR", runtime.to_string_lossy().into_owned())]);
        assert_eq!(
            pairs(&env, Some(&runtime), &x11),
            vec![
                ("WAYLAND_DISPLAY".to_owned(), "wayland-0".to_owned()),
                ("DISPLAY".to_owned(), ":0".to_owned()),
            ]
        );
    }

    /// **A display already named is left alone, and a box with no sockets
    /// is handed nothing** — the headless engine's case.
    #[test]
    fn a_named_display_is_kept_and_a_bare_box_gets_nothing() {
        let tmp = tempfile::tempdir().expect("a tempdir");
        let x11 = tmp.path().join("x11");
        std::fs::create_dir_all(&x11).expect("x11");
        sockets(&x11, &["X7"]);
        let named = Env::from_pairs([("DISPLAY", ":9")]);
        assert!(pairs(&named, None, &x11).is_empty());
        let wayland = Env::from_pairs([("WAYLAND_DISPLAY", "wayland-3")]);
        assert!(pairs(&wayland, None, &x11).is_empty());
        let bare = Env::from_pairs([("HOME", "/home/u")]);
        assert!(pairs(&bare, None, &tmp.path().join("missing")).is_empty());
        assert_eq!(
            pairs(&bare, Some(&tmp.path().join("missing")), &x11),
            vec![("DISPLAY".to_owned(), ":7".to_owned())]
        );
    }

    /// **The box's own read** goes through the runtime dir the environment
    /// names; a stand-in with one Wayland socket answers that pair whatever
    /// X11 the box has.
    #[test]
    fn discovery_reads_the_runtime_dir_the_environment_names() {
        let tmp = tempfile::tempdir().expect("a tempdir");
        sockets(tmp.path(), &["wayland-5"]);
        let env = Env::from_pairs([("XDG_RUNTIME_DIR", tmp.path().to_string_lossy().into_owned())]);
        let found = discover(&env);
        assert!(
            found.contains(&("WAYLAND_DISPLAY".to_owned(), "wayland-5".to_owned())),
            "{found:?}"
        );
    }
}
