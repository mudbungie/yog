//! **What a named state IS** — the declarative vocabulary a recipe is written
//! in, and nothing that touches disk ([`super::lay`] is the writer).
//!
//! A recipe is `&'static` data on purpose. The whole promise of a fixture world
//! is that two runs of one name lay the same bytes, and data that cannot be
//! computed cannot drift: every string a state contains is in the binary, and
//! the only reading from outside is the [`Recipe::origin`] the caller stamps
//! it with (see [`super::Laid::origin`] for why one exists at all).
//!
//! The vocabulary is deliberately small — a conversation, its marks, and the
//! shape of its newest step — because it is not a modelling language. It is the
//! set of on-disk facts the §7.3 wound, the orphaned tail, the §3.5 liveness
//! classifier and the §11 roster actually read, and a seventh arm should be
//! added only when a rendered fact has no spelling here.

/// The newest step's shape — the one fact that decides a resting
/// conversation's §3.5 state, its §7.3 wound, and whether it reads as
/// truncated. Every arm below is a real reading of
/// [`terminal::settled`](crate::git_tree) plus
/// [`wound::read`](crate::steps_view), never a flag yog stores.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// No step directory at all. `Stopped`, with nothing on disk saying a call
    /// was even attempted — which is the honest reading, not a wound.
    Absent,
    /// A `finish`-then-`end` tail: `Quiescent`.
    Settled,
    /// An `error`-then-`end` tail: `Stopped`, and no wound (the step answered,
    /// badly).
    Failed,
    /// `{"type":"finish","reason":"length"}` before the `end` — the §4.4
    /// output-limit ending, which paints `Complete` framing over a turn that
    /// did not end.
    OutputLimit,
    /// Deltas with **no terminator**: bytes came back and the segment never
    /// closed. At rest that is `Stopped`; with the step's fds held open by a
    /// harness ([`super::Laid::hold`]) it is the `InFlight` a live model call
    /// reads as.
    Streaming,
    /// **The §7.3 wound**: no `response.json` bytes and no `meta.json`, so the
    /// call emitted nothing and the step never settled. The payload is the
    /// step's `stderr.log` — empty for `Wound::Mute`, bytes for `Wound::Spoke`.
    Wound(&'static str),
}

/// One conversation to lay: the `agents/<id>` branch, its worktree files, its
/// `refs/litany/*` marks and its newest step.
pub struct Conv {
    /// The agent id — the branch leaf, the `steps/`/`inbox/` key, and the
    /// worktree directory name.
    pub id: &'static str,
    /// `goal.md`. A leading `Ball <id>: <title>` is the §3.3 stamp the
    /// conversation badge derives from.
    pub goal: &'static str,
    /// `refs/litany/<mark>/<id>` marks to point at this branch's tip:
    /// `notify`, `conflicted`, `budget-exhausted`, `abandoned`.
    pub marks: &'static [&'static str],
    /// How long before the recipe's origin this conversation last acted. The
    /// dispatch commit, the messages and the step are all dated from it, so
    /// the §11 roster's one sort key is the recipe's choice and never the
    /// laying machine's clock.
    pub age_secs: i64,
    /// The newest step's shape.
    pub step: Step,
    /// `agents/<id>/messages/<name>` entries, in the order written. `NNN-*.md`
    /// is a delivered deposit, `NNN-*.json` a model turn; a `.md` newest entry
    /// is the orphaned-**mail** tail, and a `.json` one whose `tool_use` has no
    /// answering `tool_result` is the orphaned **tool-window** tail.
    pub messages: &'static [(&'static str, &'static str)],
    /// `agents/<id>/summary/<name>` compaction summaries. A recipe that starts
    /// its `messages` above `001`, or leaves a hole, needs one of these for the
    /// transcript to render the compacted marker rather than a gap.
    pub summaries: &'static [(&'static str, &'static str)],
    /// Undelivered `inbox/<id>/<name>.md` deposits — the `✉n` count.
    pub deposits: &'static [(&'static str, &'static str)],
    /// `steps/<id>/driver.log`, the orphaned tail's reason. Empty writes no
    /// file, which is the `Mute` arm of that banner.
    pub driver_log: &'static str,
}

impl Conv {
    /// A conversation with a goal and nothing else — every other field empty,
    /// which is the general shape rather than a bootstrap case.
    pub const fn new(id: &'static str, goal: &'static str) -> Self {
        Self {
            id,
            goal,
            marks: &[],
            age_secs: 0,
            step: Step::Absent,
            messages: &[],
            summaries: &[],
            deposits: &[],
            driver_log: "",
        }
    }
}

/// One workspace to lay under the world's litany home, and the conversations
/// in it. The name is the §3.1 chosen name — the directory leaf, the wall key
/// and the registration a wire client is scoped by.
pub struct Wsp {
    pub name: &'static str,
    pub convs: &'static [Conv],
}

/// A whole named state: the workspaces, plus the settings-shaped files that
/// live beside them rather than inside any one workspace.
pub struct Recipe {
    /// The one line `yog fixture` prints for this name.
    pub summary: &'static str,
    pub workspaces: &'static [Wsp],
    /// `<yog-state-root>/cadence.yaml` — the §7.2 tuned periods, which are the
    /// settings surface with no workspace of its own.
    pub cadence: Option<&'static str>,
    /// `<world>/walls/<workspace>/brazen/config.toml` for every workspace in
    /// this recipe. **Provider rows only, never a credential**: a wall that
    /// carries a sign-in is a secret in a fixture, and the states here are for
    /// rendering, not for a turn that runs.
    pub brazen: Option<&'static str>,
}

impl Recipe {
    /// An empty recipe — the first-run state, and the base every other is
    /// written as a departure from.
    pub const fn empty(summary: &'static str) -> Self {
        Self {
            summary,
            workspaces: &[],
            cadence: None,
            brazen: None,
        }
    }
}
