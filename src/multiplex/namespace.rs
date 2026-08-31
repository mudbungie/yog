//! The multiplex router's namespace table and its exhaustive classification
//! (§16.7 W12, bl-2930; `owns_argv` bl-4667) — one home for which words route
//! to an arm, and which of those arms own their own argv.

/// Every namespace and its arm — the router's whole table (§16.7 W12,
/// bl-2930). What a word *means* is [`super::help::COMMANDS`]'s business; balls' two
/// plugin binaries are absent from it because balls' own plugin chain spawns
/// them and no operator types one.
pub(super) const NAMESPACES: &[(&str, Namespace)] = &[
    ("litany", Namespace::Litany),
    ("bl", Namespace::Bl),
    ("bz", Namespace::Bz),
    ("bl-delivery", Namespace::BlDelivery),
    ("bl-tracker", Namespace::BlTracker),
    ("gesture", Namespace::Gesture),
];

/// The embedded-tool namespaces yog multiplexes to (§16.7 W12): the three
/// agent tools, plus balls' two sibling plugin binaries (bl-2930) — spawned
/// not by yog but by the embedded balls' own plugin chain, through the
/// `world/tools/` shims a `yog bl prime` binds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Namespace {
    Litany,
    Bl,
    Bz,
    BlDelivery,
    BlTracker,
    Gesture,
}

impl Namespace {
    /// The namespace a leading verb names, or `None` — the signal that `arg` is
    /// not a multiplex target (the hatch/boot path).
    pub(super) fn from_arg(arg: &str) -> Option<Self> {
        NAMESPACES
            .iter()
            .find(|(word, _)| *word == arg)
            .map(|(_, namespace)| *namespace)
    }

    /// Whether this namespace's argv **belongs to the tool behind it**, so a
    /// `--help` falls through to the arm and the tool prints its own page —
    /// the embedded tools, balls' two plugin seams, and the gesture grammar.
    /// Every namespace left after the severance (bl-7942) owns its argv; the
    /// two that did not were the wire's client modes, and they are the seat
    /// crate's. The match is exhaustive on purpose: an added variant cannot
    /// compile unclassified — which is how bl-4667 caught the last one that
    /// conflated "routes as a namespace" with "owns its argv" and regressed
    /// bl-52ed's every-command-answers invariant.
    pub(super) fn owns_argv(self) -> bool {
        match self {
            Namespace::Litany
            | Namespace::Bl
            | Namespace::Bz
            | Namespace::BlDelivery
            | Namespace::BlTracker
            | Namespace::Gesture => true,
        }
    }

    /// Route to the namespace's arm with the sliced verb args (§16.7 W12).
    pub(super) fn run(self, args: &[String]) -> i32 {
        match self {
            Namespace::Litany => super::litany::run(args),
            Namespace::Bl => super::bl::run(args),
            Namespace::Bz => super::bz::run(args),
            Namespace::BlDelivery => super::bl_delivery::run(args),
            Namespace::BlTracker => super::bl_tracker::run(args),
            Namespace::Gesture => super::gesture::run(args),
        }
    }
}
