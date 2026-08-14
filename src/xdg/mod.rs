//! The one home for every filesystem-path derivation in yog (DESIGN §15 Y2,
//! §5.1). Balls, lernie and yog locate their state through XDG folds; this
//! module reproduces each fold once, over an injected [`Env`] snapshot —
//! except **balls'**, which is no longer reproduced at all: the crate is
//! linked (§16.7 W8), so [`Env::balls_layout`] hands back balls' own
//! `layout::Xdg` and every balls path derives from it.
//!
//! **Brazen's folds are not here, and no longer XDG at all.** Since the
//! blast-radius ruling (§16.2) brazen's config, credentials and
//! model cache resolve inside the focused workspace's *wall*
//! ([`crate::world::wall`], [`Env::wall`]) — one yog-owned layout, identical on
//! every §10 target, so the per-OS branch this module used to carry for them
//! went with the sharing it served.
//!
//! The discipline that makes it testable: **no fold reads the process
//! environment.** They read only the [`Env`] handed to them, so a test drives
//! every branch with a hermetic env and Linux tarpaulin covers all of them —
//! and there is no per-OS arm left here to cover, the brazen folds that once
//! carried one having gone per-wall (above). The sole bridge to the real
//! environment is [`Env::from_env`]; nothing else in the crate may read env
//! for paths.

use balls::layout::Xdg as BallsXdg;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// An immutable snapshot of the environment variables the folds consult.
/// Constructed once from the process (`from_env`) or explicitly (`from_pairs`).
#[derive(Debug, Clone)]
pub struct Env {
    vars: HashMap<String, String>,
}

impl Env {
    /// Snapshot the live process environment. The only env read in the crate.
    pub fn from_env() -> Self {
        Self {
            vars: std::env::vars().collect(),
        }
    }

    /// Build from explicit pairs — the hermetic path for tests (the sole
    /// non-`from_env` constructor; production reads the process env once).
    #[cfg(test)]
    pub(crate) fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut vars = HashMap::new();
        for (k, v) in pairs {
            vars.insert(k.into(), v.into());
        }
        Self { vars }
    }

    /// Derive a new snapshot from this one with the given `(key, value)` pairs
    /// overridden — inserted, or replaced when already present. The composition
    /// seam for the nested world (§16.2, [`crate::world`]): the world layers its
    /// fixed override set (`LERNIE_HOME`/`XDG_STATE_HOME`) over
    /// the ambient snapshot, and the result is itself an `Env`, so every fold
    /// above re-derives through the world with no second code path. A concrete
    /// slice (not `impl IntoIterator`) keeps the seam off the crate's generic
    /// public surface; `pub(crate)` — an internal composition point, not API.
    pub(crate) fn with_overrides(&self, overrides: &[(&str, &str)]) -> Env {
        let mut vars = self.vars.clone();
        for &(k, v) in overrides {
            vars.insert(k.to_owned(), v.to_owned());
        }
        Env { vars }
    }

    /// Derive a snapshot with `key` absent. The wall's other direction
    /// ([`crate::world::wall::env_opt`]): an unfocused surface must not inherit
    /// the last focused workspace's wall, and "absent" is the same emptiness a
    /// process outside any workspace already reads. `pub(crate)` — a
    /// composition point, not API.
    pub(crate) fn without(&self, key: &str) -> Env {
        let mut vars = self.vars.clone();
        vars.remove(key);
        Env { vars }
    }

    /// A present, non-empty variable, owned. The one read the wall's ambient
    /// credential discovery needs ([`crate::bz_host::store`]: an `ApiKeyEnv`
    /// spec names a variable whose *value* is the key), taken from the injected
    /// snapshot rather than the live process env like every other fold here.
    pub(crate) fn var(&self, key: &str) -> Option<String> {
        self.get(key).map(str::to_owned)
    }

    /// The focused workspace's wall root, named by
    /// [`YOG_WALL`](crate::world::wall::YOG_WALL) (§16.2 as amended). `None`
    /// when the snapshot names no wall — a seat inside no workspace, which has
    /// no providers, no credentials and no model cache to read: nothing is
    /// ambient but the roster of workspaces itself.
    pub fn wall(&self) -> Option<PathBuf> {
        self.get(crate::world::wall::YOG_WALL).map(PathBuf::from)
    }

    /// Every variable in the snapshot, as owned pairs. The bridge to a linked
    /// crate's own env-snapshot type (§16.7 W10: `brazen::EnvSnapshot`), so an
    /// embedded tool folds its config through the *same* snapshot yog's own
    /// folds read — one env, one answer, no second code path. `pub(crate)`: a
    /// composition seam, not API.
    pub(crate) fn pairs(&self) -> Vec<(String, String)> {
        self.vars
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// A present, non-empty variable. Empty reads as absent — the XDG
    /// convention, and lernie's `LERNIE_HOME` "empty falls through" semantics,
    /// unified into one rule.
    fn get(&self, key: &str) -> Option<&str> {
        self.vars
            .get(key)
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }

    /// `$HOME/<tail>`, or a bare relative `<tail>` when HOME is absent.
    fn home(&self, tail: &str) -> PathBuf {
        match self.get("HOME") {
            Some(h) => PathBuf::from(h).join(tail),
            None => PathBuf::from(tail),
        }
    }

    /// `$<var>/<sub>` when `var` is set, else `$HOME/<default_tail>/<sub>`.
    fn xdg(&self, var: &str, default_tail: &str, sub: &str) -> PathBuf {
        let base = match self.get(var) {
            Some(base) => PathBuf::from(base),
            None => self.home(default_tail),
        };
        base.join(sub)
    }

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

    /// The operator's home dir (`$HOME`), else the filesystem root `/` — the bare
    /// rung's driver cwd (`~`, §3.4). Unset HOME falls back to a real, always-present
    /// directory, never the empty path that would spawn the loop into cwd `""`. Read
    /// from the snapshot, never the live env.
    pub fn home_dir(&self) -> PathBuf {
        self.get("HOME")
            .map_or_else(|| PathBuf::from("/"), PathBuf::from)
    }

    /// Yog data root: `$XDG_DATA_HOME/yog` else `~/.local/share/yog`.
    pub fn yog_data_root(&self) -> PathBuf {
        self.xdg("XDG_DATA_HOME", ".local/share", "yog")
    }

    /// Yog state root: `$XDG_STATE_HOME/yog` else `~/.local/state/yog`.
    pub fn yog_state_root(&self) -> PathBuf {
        self.xdg("XDG_STATE_HOME", ".local/state", "yog")
    }

    /// The invoking user (`$USER`) — the default claim identity when `ui.json`
    /// records none (§4.1 `identity_last_used`: "default `$USER` when absent").
    /// Read from the snapshot, never the live process env (the module rule).
    pub fn user(&self) -> Option<String> {
        self.get("USER").map(str::to_owned)
    }

    /// The ambient executable search path (`$PATH`), or `None` when unset or
    /// empty. The one env fact the world's tools override prepends to (§16.2,
    /// §16.7 W9): `<world>/tools` goes in front of this value so an agent's
    /// bare `bl` resolves to yog's own shim. Read from the snapshot, never the
    /// live process env (the module rule).
    pub fn search_path(&self) -> Option<String> {
        self.get("PATH").map(str::to_owned)
    }

    /// The workspace root lernie's tool-control seam names when it consults the
    /// capability control (`LERNIE_CONV_REPO`, §8.6). `None` when unset — the
    /// control is also *run* in that directory, so the caller's fallback is the
    /// same fact by its other spelling. Read from the snapshot, never the live
    /// process env (the module rule).
    pub fn lernie_conv_repo(&self) -> Option<PathBuf> {
        self.get("LERNIE_CONV_REPO").map(PathBuf::from)
    }

    /// Yog scripted-editor staging root: `<yog-state-root>/stage` (§9.3,
    /// §5.2). The per-edit `<nonce>/` dirs live under it; leftovers older
    /// than 24 h are swept at startup.
    pub fn yog_stage_root(&self) -> PathBuf {
        stage_root_under(&self.yog_state_root())
    }
}

/// The [`Env`]-free core [`Env::yog_stage_root`] delegates to: the staging root
/// under a yog **state** root (§9.3, §5.2). The start flow already carries that
/// root (it is where `ops.jsonl` is appended), so its §8.1 config write derives
/// the staging dir through here rather than re-snapshotting the process env.
pub fn stage_root_under(yog_state_root: &Path) -> PathBuf {
    yog_state_root.join("stage")
}

/// A hex nibble, or `None` for a non-hex byte.
fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Hand-rolled `%XX` decoder for balls clone-dir names. An invalid escape
/// (`%` at the end, or not followed by two hex digits) passes through
/// verbatim rather than erroring; the inverse is balls' concern, not ours.
pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    // `get` at each offset both bounds-checks and reads: the loop ends when the
    // current byte is absent, and a short/invalid `%XX` tail falls through.
    while let Some(&cur) = b.get(i) {
        if cur == b'%'
            && let (Some(&hi), Some(&lo)) = (b.get(i + 1), b.get(i + 2))
            && let (Some(h), Some(l)) = (hex_val(hi), hex_val(lo))
        {
            out.push((h << 4) | l);
            i += 3;
            continue;
        }
        out.push(cur);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests;
