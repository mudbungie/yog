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
    (crate::wire::SEAT_SUBCMD, Namespace::Seat),
    (crate::wire::HOST_SUBCMD, Namespace::ToolHost),
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
    Seat,
    ToolHost,
}

impl Namespace {
    /// The namespace a leading verb names, or `None` — the signal that `arg` is
    /// not a multiplex target (the GUI/hatch path).
    pub(super) fn from_arg(arg: &str) -> Option<Self> {
        NAMESPACES
            .iter()
            .find(|(word, _)| *word == arg)
            .map(|(_, namespace)| *namespace)
    }

    /// Whether this namespace's argv **belongs to the tool behind it**, so a
    /// `--help` falls through to the arm and the tool prints its own page —
    /// the embedded tools, balls' two plugin seams, the gesture grammar, and
    /// `seat` (whose payload *is* that grammar and which answers `Query::Help`
    /// itself, engine-free — `wire/seat.rs`). `ToolHost` routes as a namespace
    /// but takes no argv at all, so it has no interface of its own to consult
    /// and its page is [`super::help::COMMANDS`]'s to answer, like `serve`'s
    /// (bl-4667 — conflating "routes as a namespace" with "owns its argv" is
    /// exactly how bl-52ed's every-command-answers invariant regressed). The
    /// match is exhaustive on purpose: an added variant cannot compile
    /// unclassified.
    pub(super) fn owns_argv(self) -> bool {
        match self {
            Namespace::Litany
            | Namespace::Bl
            | Namespace::Bz
            | Namespace::BlDelivery
            | Namespace::BlTracker
            | Namespace::Gesture
            | Namespace::Seat => true,
            Namespace::ToolHost => false,
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
            Namespace::Seat => super::wire::seat(args),
            Namespace::ToolHost => super::wire::tool_host(args),
        }
    }
}
