use super::*;

/// A fold under test, a hermetic env as pairs, and the path it must yield.
type Case = (
    fn(&Env) -> PathBuf,
    &'static [(&'static str, &'static str)],
    &'static str,
);

fn run(cases: &[Case]) {
    for &(fold, vars, expect) in cases {
        let env = Env::from_pairs(vars.iter().copied());
        assert_eq!(fold(&env), PathBuf::from(expect), "vars={vars:?}");
    }
}

#[test]
fn xdg_folds_set_unset_and_missing_home() {
    run(&[
        // balls: XDG set, XDG unset (HOME set), both missing.
        (
            Env::balls_state_root,
            &[("XDG_STATE_HOME", "/x")],
            "/x/balls",
        ),
        (
            Env::balls_state_root,
            &[("HOME", "/h")],
            "/h/.local/state/balls",
        ),
        // No HOME: balls' own fold roots on `home_dir()`'s `/` fallback (§16.7
        // W8 — the balls folds are `balls::layout`'s now, not yog's mirror).
        (Env::balls_state_root, &[], "/.local/state/balls"),
        (
            Env::balls_clones_dir,
            &[("XDG_STATE_HOME", "/x")],
            "/x/balls/clones",
        ),
        (
            Env::balls_clones_dir,
            &[("HOME", "/h")],
            "/h/.local/state/balls/clones",
        ),
        // yog data + state.
        (Env::yog_data_root, &[("XDG_DATA_HOME", "/d")], "/d/yog"),
        (Env::yog_data_root, &[("HOME", "/h")], "/h/.local/share/yog"),
        (Env::yog_data_root, &[], ".local/share/yog"),
        (Env::yog_state_root, &[("XDG_STATE_HOME", "/x")], "/x/yog"),
        (
            Env::yog_state_root,
            &[("HOME", "/h")],
            "/h/.local/state/yog",
        ),
        // yog scripted-editor staging root (§9.3): state root + `stage`.
        (
            Env::yog_stage_root,
            &[("XDG_STATE_HOME", "/x")],
            "/x/yog/stage",
        ),
        (
            Env::yog_stage_root,
            &[("HOME", "/h")],
            "/h/.local/state/yog/stage",
        ),
        // home dir (`~`, §3.4 bare rung cwd): HOME verbatim, else the root `/`
        // (a real dir, never cwd "").
        (Env::home_dir, &[("HOME", "/h")], "/h"),
        (Env::home_dir, &[], "/"),
    ]);
}

#[test]
fn litany_roots_home_collapse_set_empty_unset() {
    run(&[
        // LITANY_HOME set and non-empty: both roots collapse onto it,
        // even with XDG vars present.
        (
            Env::litany_config_root,
            &[("LITANY_HOME", "/L"), ("XDG_CONFIG_HOME", "/c")],
            "/L",
        ),
        (
            Env::litany_data_root,
            &[("LITANY_HOME", "/L"), ("XDG_DATA_HOME", "/d")],
            "/L",
        ),
        // Empty LITANY_HOME falls through to the XDG fold.
        (
            Env::litany_config_root,
            &[("LITANY_HOME", ""), ("XDG_CONFIG_HOME", "/c")],
            "/c/litany",
        ),
        (
            Env::litany_data_root,
            &[("LITANY_HOME", ""), ("XDG_DATA_HOME", "/d")],
            "/d/litany",
        ),
        // Unset LITANY_HOME: XDG, then HOME default, then bare relative.
        (
            Env::litany_config_root,
            &[("XDG_CONFIG_HOME", "/c")],
            "/c/litany",
        ),
        (
            Env::litany_config_root,
            &[("HOME", "/h")],
            "/h/.config/litany",
        ),
        (Env::litany_config_root, &[], ".config/litany"),
        (
            Env::litany_data_root,
            &[("HOME", "/h")],
            "/h/.local/share/litany",
        ),
        (Env::litany_data_root, &[], ".local/share/litany"),
    ]);
}

#[test]
fn percent_decode_table() {
    let cases = [
        ("home%2Fmark", "home/mark"), // uppercase hex
        ("home%2fmark", "home/mark"), // lowercase hex
        ("a%20b", "a b"),             // digits
        ("plain", "plain"),           // no escape
        ("", ""),                     // empty
        ("%GG", "%GG"),               // invalid hex, verbatim
        ("tail%", "tail%"),           // trailing %, verbatim
        ("x%4", "x%4"),               // incomplete escape, verbatim
        ("%41%42", "AB"),             // consecutive escapes
    ];
    for (input, expect) in cases {
        assert_eq!(percent_decode(input), expect, "input={input:?}");
    }
}

#[test]
fn user_reads_the_snapshot_and_empty_is_absent() {
    assert_eq!(
        Env::from_pairs([("USER", "orion")]).user().as_deref(),
        Some("orion"),
    );
    // Empty and unset both read as absent (the module's `get` convention).
    assert_eq!(Env::from_pairs([("USER", "")]).user(), None);
    assert_eq!(
        Env::from_pairs(std::iter::empty::<(&str, &str)>()).user(),
        None
    );
}

#[test]
fn from_env_snapshots_process_env() {
    // Exercises the one real-env bridge; contents are host-dependent, so we
    // only assert it constructs and folds without panicking.
    let env = Env::from_env();
    let _ = env.yog_state_root();
}

/// **Both clone roots, and the one they collapse to** (§5.1 #1, bl-262a).
///
/// Inside a composed world the two differ — the world's own bundle and the
/// host's — because a project is one balls invocation path and the store for a
/// directory is the directory's. Outside one they are the same path and the
/// pair is a single root: nothing to enumerate twice, and no case to spell.
#[test]
fn the_clone_roots_are_the_worlds_and_the_hosts_or_the_one_they_collapse_to() {
    let ambient = Env::from_pairs([
        ("HOME", "/h"),
        ("XDG_DATA_HOME", "/d"),
        ("XDG_STATE_HOME", "/s"),
    ]);
    assert_eq!(
        ambient.balls_clone_roots(),
        vec![PathBuf::from("/s/balls/clones")],
        "outside a world there is one bundle"
    );
    assert_eq!(
        crate::world::compose(&ambient).balls_clone_roots(),
        vec![
            PathBuf::from("/d/yog/world/state/balls/clones"),
            PathBuf::from("/s/balls/clones"),
        ],
        "inside one, the world's own and the host's — world first, since a \
         path present in both is one project and the first reading wins"
    );
}

/// The host's state home is the ambient reading until the world carries one in,
/// and then it is the one the world carried (bl-262a) — which is what makes the
/// fold survive the very override that hides it.
#[test]
fn the_host_state_home_is_the_ambient_reading_the_world_carried_in() {
    let ambient = Env::from_pairs([("HOME", "/h"), ("XDG_STATE_HOME", "/s")]);
    assert_eq!(ambient.host_state_home(), PathBuf::from("/s"));
    assert_eq!(
        Env::from_pairs([("HOME", "/h")]).host_state_home(),
        PathBuf::from("/h/.local/state"),
        "with no XDG_STATE_HOME it is balls' own fallback, off HOME"
    );
    let world = crate::world::compose(&ambient);
    assert_ne!(world.host_state_home(), world.balls_state_home());
    assert_eq!(world.host_state_home(), PathBuf::from("/s"));
}
