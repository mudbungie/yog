//! The §11 **center tabs** — the vocabulary of what the window's middle column
//! is showing right now.
//!
//! Sibling of [`InspectorTab`](super::InspectorTab) and split on the same seam:
//! everything here is read by the shell's tab strip, and only the digit map is
//! a keymap fact. The two strips are different altitudes — the inspector's
//! tabs are views of *one conversation* (altitude 2), these are the peers the
//! whole center can be (altitude 1) — so they are two enums, not one.
//!
//! **Why there is an enum here at all** (bl-1ca2). Several surfaces — config
//! among them — were interface overlays toggled on rather than tabs, and since
//! they cover everything they should simply be tab focuses. Config, Login and the world-search results each used to be a mode
//! toggled on *over* the conversation — a `bool` apiece, none of them aware of
//! the others. One enum is what makes them peers: the center shows exactly one
//! of these, always, and "leave Config" is not a verb but the ordinary act of
//! focusing another tab.

/// What the center column is showing (§11 altitude 1). Exactly one at a time,
/// per-instance viewport ephemera (§13.1) held in the shell's RAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CenterTab {
    /// The selected conversation and its altitude-2 inspector — the home tab,
    /// and the only one with a composer, which is why every focus request
    /// lands here (`shell/focus.rs`).
    #[default]
    Conversation,
    /// The §9 write surfaces: brazen's `config.toml`, lernie's global config,
    /// this workspace's config branches.
    Config,
    /// The §8.3 `bz --login` device flow, one row per provider.
    Login,
    /// The §8.5 world-search results. Offered only while there is an answer to
    /// show — the tab is a view of the published answer, not a mode to enter.
    Search,
}

impl CenterTab {
    /// The tab a **Command+Shift+digit** press selects (§11 tab order): 1
    /// Conversation … 4 Search. The plain Command+digit plane belongs to the
    /// altitude-2 inspector, so the center's strip — the *other* tab strip —
    /// takes the shifted plane, exactly as `new workspace` takes the "other
    /// new". Digits outside 1–4 select no tab.
    pub fn from_digit(n: u8) -> Option<Self> {
        Some(match n {
            1 => Self::Conversation,
            2 => Self::Config,
            3 => Self::Login,
            4 => Self::Search,
            _ => return None,
        })
    }

    /// The digit that selects this tab, on the shifted plane — its place in
    /// [`all`](Self::all), so the map and its inverse are one list (bl-478d).
    pub fn digit(self) -> u8 {
        let at = Self::all().iter().position(|tab| *tab == self);
        u8::try_from(at.unwrap_or_default() + 1).unwrap_or(1)
    }

    /// The tab's strip label (§11).
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Conversation => "Conversation",
            Self::Config => "Config",
            Self::Login => "Login",
            Self::Search => "Search",
        }
    }

    /// What focusing this tab shows, in operator terms — exhaustive over the
    /// enum, so a tab cannot ship without saying what it is (§11
    /// discoverability invariant). The words live here rather than at the
    /// strip because the left-panel entries name the same tabs and must not
    /// grow a second phrasing of them.
    pub(crate) fn hint(self) -> &'static str {
        match self {
            Self::Conversation => {
                "The conversation you have selected — its transcript, its inspector and \
                 the box you talk to it in. Escape from any other tab comes back here."
            }
            Self::Config => {
                "Every setting yog can write: brazen's config.toml, lernie's global \
                 config, and this workspace's config branches. Focusing the tab re-reads \
                 all three off disk; nothing is written until you Apply."
            }
            Self::Login => {
                "Sign in to a model provider: one button per row, each opening your \
                 browser to authorize it and printing everything bz says."
            }
            Self::Search => {
                "What the last /search found across the whole world — balls, workspaces \
                 and conversations. Pick a result to go there."
            }
        }
    }

    /// The hover a control that **focuses** this tab wears: what the tab shows,
    /// then the §11 combo that presses it without the mouse (bl-478d rule 3).
    /// One home, because three seats ask for it — the centre strip, the
    /// navigator's two entries, and the §9.4 picker's route out of a credential
    /// fault (bl-91f1) — and three phrasings of one gesture is two too many.
    pub(crate) fn focus_hover(self) -> String {
        format!(
            "{} Press Ctrl+Shift+{}: the centre strip is Ctrl+Shift+1 to \
             Ctrl+Shift+4.",
            self.hint(),
            self.digit()
        )
    }

    /// The tabs in §11 strip order — the strip and the digit map both derive
    /// from this one list.
    pub fn all() -> [Self; 4] {
        [Self::Conversation, Self::Config, Self::Login, Self::Search]
    }
}
