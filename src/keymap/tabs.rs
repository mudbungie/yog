//! The §11 Altitude-2 inspector's tab vocabulary — the enum, its digit map,
//! its labels, and which tabs a rail pin reaches. Split out of [`super`] at
//! §12's per-file budget, on the seam between *what the keys mean* and *what
//! the tabs are*: everything here is read by the inspector and by the shell's
//! tab strip, and only its digit map is a keymap fact.

/// The §11 Altitude-2 inspector tabs (per selected agent). The digit keys
/// select these; the tab is per-instance viewport ephemera (§5.3), held in RAM
/// on [`Focus`](crate::Focus).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    #[default]
    Transcript,
    Steps,
    Inbox,
    Files,
    Config,
    /// What this workspace's attempts changed in their project (§5.1 #32) —
    /// last, because it is the only tab whose subject is the project repo
    /// rather than the conversation.
    Work,
}

impl InspectorTab {
    /// The tab a digit key selects (§11 tab order): 1 Transcript … 6 Work.
    /// Digits outside 1–6 select no tab.
    pub fn from_digit(n: u8) -> Option<Self> {
        Some(match n {
            1 => Self::Transcript,
            2 => Self::Steps,
            3 => Self::Inbox,
            4 => Self::Files,
            5 => Self::Config,
            6 => Self::Work,
            _ => return None,
        })
    }

    /// The digit that selects this tab — its place in [`all`](Self::all), so the
    /// map and its inverse are one list and a hover naming the key cannot drift
    /// from the key that works (bl-478d).
    pub fn digit(self) -> u8 {
        let at = Self::all().iter().position(|tab| *tab == self);
        u8::try_from(at.unwrap_or_default() + 1).unwrap_or(1)
    }

    /// The tab's header label (§11 inspector).
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Transcript => "Transcript",
            Self::Steps => "Steps",
            Self::Inbox => "Inbox",
            Self::Files => "Files",
            Self::Config => "Config",
            Self::Work => "Work",
        }
    }

    /// The six tabs in §11 order — the shell's tab strip and the digit map
    /// both derive from this one list.
    pub fn all() -> [Self; 6] {
        [
            Self::Transcript,
            Self::Steps,
            Self::Inbox,
            Self::Files,
            Self::Config,
            Self::Work,
        ]
    }

    /// Does a rail pin reach this tab? The pin is a commit of the
    /// *conversation's* repo (VISION V1.2), and every tab but [`Work`](Self::Work)
    /// reads that repo. Work reads the project repo, whose state the
    /// conversation's history does not yet index — the per-step project commit
    /// is VISION §4.10 item 4's, and it is not written yet — so the pin does
    /// not reach it and the banner does not claim it.
    pub fn pinnable(self) -> bool {
        !matches!(self, Self::Work)
    }
}
