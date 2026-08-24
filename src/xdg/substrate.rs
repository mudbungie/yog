//! **Where the embedded substrates keep their state** — balls' own
//! `layout::Xdg` over this snapshot (§16.7 W8: the crate is linked, so yog
//! reproduces no balls fold of its own) and lernie's `LERNIE_HOME` override
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
        let space = crate::world::marks::space(self);
        BallsXdg::with(
            &self.home_dir(),
            Some(&space.config.to_string_lossy()),
            Some(&space.state.to_string_lossy()),
        )
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

    /// `$LERNIE_HOME` when set collapses both lernie roots onto that dir.
    fn lernie_home(&self) -> Option<PathBuf> {
        self.get("LERNIE_HOME").map(PathBuf::from)
    }

    /// Lernie config root: `$LERNIE_HOME` else `$XDG_CONFIG_HOME/lernie` else
    /// `$HOME/.config/lernie`.
    pub fn lernie_config_root(&self) -> PathBuf {
        self.lernie_home()
            .unwrap_or_else(|| self.xdg("XDG_CONFIG_HOME", ".config", "lernie"))
    }

    /// Lernie data root: `$LERNIE_HOME` else `$XDG_DATA_HOME/lernie` else
    /// `$HOME/.local/share/lernie`.
    pub fn lernie_data_root(&self) -> PathBuf {
        self.lernie_home()
            .unwrap_or_else(|| self.xdg("XDG_DATA_HOME", ".local/share", "lernie"))
    }
}
