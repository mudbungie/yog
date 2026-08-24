//! The Login pane's surface RAM (DESIGN §5.3, §8.3) — its own file per §12's
//! line budget, and a real seam: everything else in [`super`] is viewport
//! ephemera, while this holder owns a *wall-bound* seam bundle (the in-process
//! `bz` runner, the credentials dir a presence read folds against, and the
//! spawn layer a sign-in is fired with, §16.2 as amended) — so it is one field
//! of [`WallRam`](super::WallRam), one holder per workspace.

use std::path::PathBuf;

use crate::xdg::Env;

/// The Login pane's surface RAM (§5.3, §8.3): the runner that answers
/// brazen's effective provider table in-process (§16.7 W10), the **rendered
/// rows** that table and the credential-presence read last derived to
/// (#20/#21/#22), the credentials dir that read folds against, and the one
/// active streamed `bz --login` run, if any. Instance-local by nature — a
/// device code is for the human at *this* keyboard — and discarded on exit.
pub struct LoginHolder {
    pub bz: crate::config_edit::brazen::RealBzRunner,
    pub rows: Vec<crate::config_edit::brazen::ProviderRowView>,
    /// **What the §8.1 start gate reads** (bl-1fd0): the same `ask`'s second
    /// product, off the same rows, so the start pane's provider rung and the
    /// roster painted under it can never disagree about this wall. Two
    /// booleans rather than a second copy of the table — the gate is the only
    /// consumer and the columns it folds are `credential` and nothing else.
    pub credit: crate::start::WallCredit,
    pub run: Option<crate::login::LoginRun>,
    /// Whether [`run`](Self::run)'s finished sign-in has already been folded
    /// back into the rows. The fold is a **gesture** read like every other
    /// (§7.2) and a settled run stays settled, so without this latch the frame
    /// after a sign-in would re-ask brazen forever.
    folded: bool,
    /// The wall layer (§16.2 as amended) the streamed `bz --login` spawn is
    /// fired with, so the credential it writes lands in *this* workspace.
    pub wall: Vec<(String, String)>,
    /// The workspace this holder belongs to — the §8.3 run-by-hand fallback's
    /// `--ws` (bl-b589). The wall *pairs* above name the wall root, and a wall
    /// root cannot be turned back into the workspace path that keyed it, so the
    /// path is carried rather than recovered.
    pub workspace: Option<PathBuf>,
    creds_dir: PathBuf,
}

impl LoginHolder {
    /// Fold every seam from `wall` — the workspace's lensed env (§16.2 as
    /// amended) this holder belongs to — **and ask** (bl-e290): the holder is
    /// never born empty, so the Login surface — the pane and the auth-failed
    /// banner alike — is populated the moment it is first painted, with no click
    /// standing between the operator and the roster.
    ///
    /// Providers, the credential column and the sign-in spawn fold together
    /// because they are one sphere's settings; outside a wall the fold is empty
    /// and the roster reads empty, which is the truth rather than the machine's
    /// own sign-ins. There is no re-seating verb: a holder is one wall's for
    /// life, and a focus change swaps the whole holder (bl-5894) — which is what
    /// keeps a running `bz --login` with the workspace it will write into.
    pub fn new(wall: &Env, workspace: Option<&std::path::Path>) -> Self {
        let mut holder = Self {
            workspace: workspace.map(std::path::Path::to_path_buf),
            bz: crate::config_edit::brazen::RealBzRunner::resolve(wall),
            rows: Vec::new(),
            credit: crate::start::WallCredit::default(),
            run: None,
            folded: false,
            wall: crate::world::wall::pairs_of(wall),
            creds_dir: crate::config_edit::brazen::BrazenPaths::of(wall)
                .map(|p| p.credentials_dir)
                .unwrap_or_default(),
        };
        holder.ask();
        holder
    }

    /// Re-derive the rendered rows: brazen's effective table
    /// ([`RealBzRunner::providers`], offline and in-process since §16.7 W10)
    /// paired with the §5.1 #22 credential-presence read, folded to words by
    /// the one shared derivation (`row_views`, bl-402f). A **gesture** read,
    /// never a per-frame one (§7.2): construction, and the pane's `↻` — which
    /// is the freshness answer for both halves at once (a config edit, and the
    /// credential a completed sign-in just wrote).
    pub fn ask(&mut self) {
        use crate::config_edit::brazen::{BzRunner, credential_presence, row_views};
        let rows = self.bz.providers();
        let creds = credential_presence(&self.creds_dir, &rows, &crate::config_edit::RealFileIo);
        self.credit = crate::start::WallCredit::read(&rows);
        self.rows = row_views(&rows, &creds);
    }

    /// Seat a freshly spawned `bz --login` as this wall's live run (§8.3) —
    /// the **one** door, so the fold latch below cannot be left set by the run
    /// before it.
    pub fn begin(&mut self, run: crate::login::LoginRun) {
        self.run = Some(run);
        self.folded = false;
    }

    /// The frame duty of a live sign-in: drain it, and **the one frame it
    /// settles clean, re-read the rows** (bl-1fd0). A completed sign-in is
    /// exactly the gesture [`ask`](Self::ask) exists for — until this, the §8.3
    /// `↻` was the only way a row the operator had just signed into started
    /// reading as signed in, and the §8.1 rung above it would have stood over a
    /// credential that already existed. Returns whether there is a run to
    /// paint at all.
    ///
    /// A non-zero exit folds nothing: the run's own lines and the run-by-hand
    /// fallback are the answer there, and re-asking brazen would only restate
    /// the state the operator is looking at.
    pub fn poll_run(&mut self) -> bool {
        let Some(live) = self.run.as_mut().map(crate::login::LoginRun::poll) else {
            return false;
        };
        if !live && !self.folded {
            self.folded = true;
            if self.run.as_ref().and_then(|r| r.view().outcome) == Some(0) {
                self.ask();
            }
        }
        true
    }
}
