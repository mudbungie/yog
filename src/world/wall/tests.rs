use super::*;
use crate::config_edit::brazen::BrazenPaths;

/// An ambient snapshot with the anchor set and a wall deliberately pre-set to
/// *another* value — so a passing derivation proves the lens **replaced** it.
fn ambient() -> Env {
    Env::from_pairs([
        ("HOME", "/h"),
        ("XDG_DATA_HOME", "/d"),
        (YOG_WALL, "/stale/wall"),
    ])
}

#[test]
fn the_wall_is_the_world_root_plus_the_workspace_leaf() {
    let world = crate::world::compose(&ambient());
    assert_eq!(walls_dir(Path::new("/w")), PathBuf::from("/w/walls"));
    assert_eq!(
        root(&world, "corp"),
        PathBuf::from("/d/yog/world/walls/corp")
    );
    assert_eq!(
        root_under(Path::new("/w"), "corp"),
        PathBuf::from("/w/walls/corp")
    );
    // A workspace path is named by its §3.1 leaf, wherever it is rooted — a
    // foreign workspace under litany's own tree gets a wall by the same fold.
    assert_eq!(
        root_of(&world, Path::new("/d/yog/workspaces/corp")),
        PathBuf::from("/d/yog/world/walls/corp")
    );
    assert_eq!(
        root_of(&world, Path::new("/elsewhere/litany/workspaces/a1b2")),
        PathBuf::from("/d/yog/world/walls/a1b2")
    );
}

#[test]
fn the_lens_replaces_a_standing_wall_and_is_idempotent() {
    let world = crate::world::compose(&ambient());
    // The pre-set (stale) value is replaced, not merely filled in.
    let corp = env(&world, Path::new("/ws/corp"));
    assert_eq!(corp.wall(), Some(PathBuf::from("/d/yog/world/walls/corp")));
    // Re-lensing an already-lensed Env swaps the sphere rather than stacking.
    let home = env(&corp, Path::new("/ws/home"));
    assert_eq!(home.wall(), Some(PathBuf::from("/d/yog/world/walls/home")));
    // …and re-lensing on the same workspace is the identity.
    assert_eq!(env(&corp, Path::new("/ws/corp")).wall(), corp.wall());
    // The anchor survives the lens, so the world stays self-consistent.
    assert_eq!(corp.yog_data_root(), world.yog_data_root());
}

#[test]
fn no_focus_is_no_wall_and_no_brazen_paths() {
    let world = crate::world::compose(&ambient());
    let none = env_opt(&world, None);
    assert_eq!(none.wall(), None);
    assert_eq!(BrazenPaths::of(&none), None);
    assert!(pairs_of(&none).is_empty());
    // …and the optional lens agrees with the plain one when there IS a focus.
    let ws = Path::new("/ws/corp");
    assert_eq!(env_opt(&world, Some(ws)).wall(), env(&world, ws).wall());
}

#[test]
fn the_spawn_layer_and_the_read_lens_name_one_path() {
    let world = crate::world::compose(&ambient());
    let ws = Path::new("/ws/corp");
    let lensed = env(&world, ws);
    // What a child is told and what this process reads are the same fact —
    // otherwise the dir yog watches and the dir a spawned `bz` writes diverge.
    assert_eq!(pairs(&world, ws), pairs_of(&lensed));
    assert_eq!(
        pairs(&world, ws),
        vec![(YOG_WALL.to_owned(), "/d/yog/world/walls/corp".to_owned())]
    );
}

#[test]
fn brazen_folds_land_inside_the_wall_and_nowhere_ambient() {
    // Every ambient brazen location is set, and none of them may win.
    let ambient = Env::from_pairs([
        ("HOME", "/h"),
        ("XDG_DATA_HOME", "/d"),
        ("XDG_CONFIG_HOME", "/cfg"),
        ("XDG_CACHE_HOME", "/cache"),
        ("BRAZEN_CONFIG", "/ambient/brazen.toml"),
    ]);
    let world = crate::world::compose(&ambient);
    let paths = BrazenPaths::of(&env(&world, Path::new("/ws/corp"))).expect("a focused wall");
    assert_eq!(
        paths,
        BrazenPaths {
            config: PathBuf::from("/d/yog/world/walls/corp/brazen/config.toml"),
            credentials_dir: PathBuf::from("/d/yog/world/walls/corp/brazen/credentials"),
            models_cache_dir: PathBuf::from("/d/yog/world/walls/corp/brazen/models"),
        }
    );
    // Two spheres share nothing: not the config, not the sign-ins, not the cache.
    let other = BrazenPaths::of(&env(&world, Path::new("/ws/home"))).expect("a focused wall");
    assert_ne!(paths.config, other.config);
    assert_ne!(paths.credentials_dir, other.credentials_dir);
    assert_ne!(paths.models_cache_dir, other.models_cache_dir);
}
