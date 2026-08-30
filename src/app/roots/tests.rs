//! The one fold from the composed world to the roots one instance walks.

use super::*;

#[test]
fn the_roots_are_folded_from_the_world_and_nothing_else() {
    let world = crate::test_support::world_under(std::path::Path::new("/w"));
    let roots = Roots::of(&world);
    assert_eq!(roots.yog_data, PathBuf::from("/w/data/yog"));
    assert_eq!(roots.litany_data, PathBuf::from("/w/litany"));
    assert_eq!(roots.yog_state, PathBuf::from("/w/state/yog"));
    assert_eq!(roots.balls_clones, PathBuf::from("/w/state/balls/clones"));
    assert_eq!(roots.home, PathBuf::from("/w/home"));
    // The four derived reads go through the same fold, so the window and the
    // windowless face cannot address different trees.
    assert_eq!(roots.names(), PathBuf::from("/w/data/yog/workspaces"));
    assert_eq!(roots.workspaces(), PathBuf::from("/w/litany/workspaces"));
    assert_eq!(roots.replays(), PathBuf::from("/w/litany/replays"));
    assert_eq!(roots.ui_json(), PathBuf::from("/w/state/yog/ui.json"));
}
