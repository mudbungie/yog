//! The scratch landing every beat here converges, and the damage it is put in.
//!
//! The damage fixture is copied from the live world rather than invented: a
//! schedule wiring only `bl-delivery`, over an OLDER balls' phase vocabulary
//! (`drop.post`, `claim.pre`, `unclaim.pre` — three keys balls 0.5.9's default
//! does not contain) and with no `show` hook at all. That is what the operator's
//! stale seed template actually produced, and it is why the repair has to
//! re-derive the whole schedule instead of patching two names into it.

use std::path::PathBuf;

use balls::edge::Edge;
use balls::substrate;
use tempfile::TempDir;
use yog::multiplex::landing::git;
use yog::world::tools;

/// The live world's damaged schedule, verbatim: only `bl-delivery`, on phase
/// keys from a retired balls, and no `show`.
const STALE: &str = r#"[hooks]
"claim.post" = ["bl-delivery"]
"claim.pre" = ["bl-delivery"]
"close.post" = ["bl-delivery"]
"close.pre" = ["bl-delivery"]
"drop.post" = ["bl-delivery"]
"prime.post" = ["bl-delivery"]
"unclaim.post" = ["bl-delivery"]
"unclaim.pre" = ["bl-delivery"]
"#;

/// A scratch world: balls' two homes, a tools dir carrying the `bl` sibling
/// roster yog's world always seeds, and a project directory to address.
pub struct World {
    _dir: TempDir,
    pub edge: Edge,
    pub landing: PathBuf,
    /// `<yog-data-root>/world` — the containment gate's subject.
    pub root: PathBuf,
}

impl World {
    /// Lay the scratch world. Nothing is founded yet — the landing path is
    /// computed by balls' own `clone_dir` fold, never spelled here.
    pub fn new() -> World {
        let dir = tempfile::tempdir().expect("scratch world");
        // The real shape: balls' two homes live INSIDE the world subtree, which
        // is what earns the repair the right to rewrite a landing there.
        let anchor = dir.path();
        let root = anchor.join("yog").join("world");
        std::fs::create_dir_all(&root).expect("world root");
        let tools = root.join("tools");
        std::fs::create_dir_all(&tools).expect("tools dir");
        // The sibling binaries balls' seed binds and prunes against. Content is
        // irrelevant — `seed::sibling` only asks whether the path exists — but
        // they must be present or the seed prunes the very entries under test.
        for name in [tools::BL, tools::BL_DELIVERY, tools::BL_TRACKER] {
            let path = tools.join(name);
            crate::write_exec::write_exec(&path, "#!/bin/sh\nexit 0\n");
        }
        let project = anchor.join("proj");
        std::fs::create_dir_all(&project).expect("project dir");
        let edge = Edge::resolve(
            anchor.to_path_buf(),
            Some(root.join("config").to_string_lossy().into_owned()),
            Some(root.join("state").to_string_lossy().into_owned()),
            project,
            Some("tester".to_owned()),
            None,
            Some(tools.join(tools::BL)),
            None,
            None,
            false,
            None,
        );
        let landing = edge.xdg.clone_dir(&edge.invocation_path).landing();
        World {
            _dir: dir,
            edge,
            landing,
            root,
        }
    }

    /// Found the landing the way `bl prime` does — balls' own seed, against
    /// this world's tools dir, so it comes up HEALTHY. **This is the fork that
    /// exiled this fixture from the lib binary** (see the crate-root note).
    pub fn found(&self) {
        substrate::found_landing(
            &self.landing,
            &self.edge.xdg,
            self.edge.exe_dir.as_deref(),
            "tester",
        )
        .expect("found landing");
    }

    /// Overwrite the founded landing's schedule with the live world's damage
    /// and commit it, reproducing a landing seeded from the stale template.
    pub fn damage(&self) {
        std::fs::write(self.plugins(), STALE).expect("write stale schedule");
        git(&self.landing, &["add", "-A"]).expect("stage");
        git(&self.landing, &["commit", "-q", "-m", "stale seed"]).expect("commit");
    }

    pub fn plugins(&self) -> PathBuf {
        self.landing.join("config").join("plugins.toml")
    }

    pub fn scalars(&self) -> PathBuf {
        self.landing.join("config").join("balls.toml")
    }

    pub fn schedule(&self) -> String {
        std::fs::read_to_string(self.plugins()).unwrap_or_default()
    }

    pub fn head(&self) -> String {
        git(&self.landing, &["rev-parse", "HEAD"]).expect("head")
    }
}
