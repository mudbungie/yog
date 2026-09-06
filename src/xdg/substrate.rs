//! **Where the embedded substrates keep their state** — balls' own
//! `layout::Xdg` over this snapshot (§16.7 W8: the crate is linked, so yog
//! reproduces no balls fold of its own) and litany's `LITANY_HOME` override
//! with the XDG folds behind it (§16.2). Split from [`super`] at §12's budget
//! on the seam that module's doc already draws: a fold that answers *where
//! another tool keeps its things* is a different subject from yog's own roots,
//! and it is the half the nested world overrides.

use super::Env;
use balls::layout::Xdg as BallsXdg;
use std::path::PathBuf;

impl Env {
    /// **balls' own** XDG layout over this snapshot (balls arch §1). Now that the
    /// crate is linked (§16.7 W8) every balls path yog derives comes from
    /// `balls::layout` — the state root, the per-invocation clone bundle, and the
    /// store checkout inside it — so yog's reads cannot drift from the layout the
    /// embedded catalog load and the multiplexed `yog bl` verbs use. Pure path
    /// arithmetic over the injected snapshot: no env reads, no IO. `$HOME` absent
    /// falls back to [`home_dir`](Self::home_dir)'s `/`, never the empty path.
    pub fn balls_layout(&self) -> BallsXdg {
        Self::layout_of(&self.home_dir(), &crate::world::marks::space(self))
    }

    /// **balls' layout for one DIRECTORY** (§16.2's one-store-per-project
    /// invariant, bl-262a) — the fold every reader with a project in hand asks,
    /// and the only one that can be right about an operator's own checkout.
    ///
    /// balls keys a clone on `(state home, invocation path)`. The world
    /// overrides the state home, so yog and the operator resolved *different*
    /// stores for the *same* directory: a conversation aimed at a project the
    /// operator tracks with `bl` read an empty store at the right path and
    /// reported the board empty, confidently and in prose. The store a
    /// directory's tasks live in is a fact of the **directory**, so the space is
    /// resolved per directory ([`marks::space_for`](crate::world::marks::space_for)):
    /// the world's own for a directory the world owns, the host's for every
    /// other.
    pub fn balls_layout_for(&self, dir: &std::path::Path) -> BallsXdg {
        Self::layout_of(&self.home_dir(), &crate::world::marks::space_for(self, dir))
    }

    /// balls' `Xdg` over one [`Space`](crate::world::marks::Space) — the two
    /// folds above meeting at their one construction, so a space and the layout
    /// it becomes cannot drift.
    fn layout_of(home: &std::path::Path, space: &crate::world::marks::Space) -> BallsXdg {
        BallsXdg::with(
            home,
            Some(&space.config.to_string_lossy()),
            Some(&space.state.to_string_lossy()),
        )
    }

    /// **The state home the operator's own shell resolves** (§16.2, bl-262a):
    /// `YOG_HOST_STATE` when the world carried it in, else this snapshot's own
    /// reading — which, outside a world, IS the operator's.
    ///
    /// The world overrides `XDG_STATE_HOME`, so once composed there is no way
    /// back to the ambient value by reading it; the world therefore carries the
    /// ambient reading forward in one var of its own
    /// ([`world::overrides`](crate::world::overrides)), idempotently, exactly as
    /// the `PATH` prepend recognizes its own entry.
    pub(crate) fn host_state_home(&self) -> PathBuf {
        match self.get(crate::world::HOST_STATE) {
            Some(base) => PathBuf::from(base),
            None => self.balls_state_home(),
        }
    }

    /// The host's config home (`$XDG_CONFIG_HOME` else `~/.config`) — needed
    /// only for the host space (bl-262a). It needs no carrier var of its own:
    /// §16.2 leaves `XDG_CONFIG_HOME` **ambient** in the world, so the value
    /// read here inside a world is already the operator's.
    pub(crate) fn host_config_home(&self) -> PathBuf {
        match self.get("XDG_CONFIG_HOME") {
            Some(base) => PathBuf::from(base),
            None => self.home_dir().join(".config"),
        }
    }

    /// balls' state home as the ambient/world snapshot resolves it —
    /// `$XDG_STATE_HOME` else `~/.local/state`, balls' own fallback, reproduced
    /// as the *input* to the §16.3 space fold rather than read back out of it
    /// (which would recurse). The world's space keeps this exactly, so every
    /// clone yog already founded is still the one it reads.
    pub(crate) fn balls_state_home(&self) -> PathBuf {
        match self.get("XDG_STATE_HOME") {
            Some(base) => PathBuf::from(base),
            // balls' own fallback anchors on [`home_dir`](Self::home_dir)'s `/`
            // when HOME is unset, never on a bare relative path.
            None => self.home_dir().join(".local/state"),
        }
    }

    /// Balls state root: `$XDG_STATE_HOME/balls` else `$HOME/.local/state/balls`
    /// — balls' own fold, via [`balls_layout`](Self::balls_layout).
    pub fn balls_state_root(&self) -> PathBuf {
        self.balls_layout().state_dir()
    }

    /// The per-project clones dir under the balls state root (balls' own fold).
    pub fn balls_clones_dir(&self) -> PathBuf {
        self.balls_layout().clones_dir()
    }

    /// **Both clone roots a project may live under** (§5.1 #1, bl-262a): the
    /// world's own and the host's, in that order and deduplicated — outside a
    /// world they are one path and the pair collapses to it.
    ///
    /// A project is one balls invocation path, and since the store for a
    /// directory is the directory's (one store per project) an operator's own
    /// checkout has its clone in the host bundle while every world-owned
    /// directory has its own in the world's. Enumerating one root would drop
    /// half the board — and, worse, would leave a bound operator project
    /// unaddressable by name, since `Snapshot::project_path` resolves over
    /// exactly this set.
    pub fn balls_clone_roots(&self) -> Vec<PathBuf> {
        let world = self.balls_clones_dir();
        let host = crate::world::marks::Space::host(self).clones_dir(&self.home_dir());
        if host == world {
            vec![world]
        } else {
            vec![world, host]
        }
    }

    /// balls' state root for one DIRECTORY (bl-262a) — the parent every
    /// `work_worktree_path` and every plugin territory folds off, resolved
    /// through that directory's own space.
    pub fn balls_state_root_for(&self, dir: &std::path::Path) -> PathBuf {
        self.balls_layout_for(dir).state_dir()
    }

    /// `$LITANY_HOME` when set collapses both litany roots onto that dir.
    fn litany_home(&self) -> Option<PathBuf> {
        self.get("LITANY_HOME").map(PathBuf::from)
    }

    /// Litany config root: `$LITANY_HOME` else `$XDG_CONFIG_HOME/litany` else
    /// `$HOME/.config/litany`.
    pub fn litany_config_root(&self) -> PathBuf {
        self.litany_home()
            .unwrap_or_else(|| self.xdg("XDG_CONFIG_HOME", ".config", "litany"))
    }

    /// Litany data root: `$LITANY_HOME` else `$XDG_DATA_HOME/litany` else
    /// `$HOME/.local/share/litany`.
    pub fn litany_data_root(&self) -> PathBuf {
        self.litany_home()
            .unwrap_or_else(|| self.xdg("XDG_DATA_HOME", ".local/share", "litany"))
    }
}
