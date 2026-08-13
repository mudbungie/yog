//! The §3.5 spend ceiling's one home in `ui.json` (DESIGN §4.1 `ceiling`) —
//! beside the price table ([`super::prices`]) it is denominated in, and read
//! exactly like it: read-only, no setter, no editor, no verb. The rates and
//! the ceiling are both operator policy with no other authority, and a hand
//! edit is live within a tick (the whole-file `adopt`, §4.1 I5).
//!
//! Absent ⇒ no ceiling ⇒ no gate. That is the severability the ruling demands
//! and it is one `get` away from being obvious.

use super::UiState;

/// The ceiling key (§4.1): the operator's spend bound in USD.
const CEILING: &str = "ceiling";

impl UiState {
    /// The §3.5 spend ceiling. Absent, or of the wrong shape, reads as *no
    /// ceiling* — deleting the key deletes the gate, not a code path.
    pub fn ceiling(&self) -> crate::spend::Ceiling {
        crate::spend::Ceiling::from_json(self.root.get(CEILING))
    }
}

#[cfg(test)]
mod tests {
    use crate::spend::Ceiling;
    use crate::ui_state::UiState;
    use tempfile::tempdir;

    fn opened(doc: &str) -> UiState {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ui.json");
        std::fs::write(&path, doc).unwrap();
        UiState::open(path)
    }

    #[test]
    fn absent_key_is_no_ceiling() {
        assert_eq!(opened(r#"{"v":1}"#).ceiling(), Ceiling::default());
    }

    #[test]
    fn wrong_shape_is_no_ceiling() {
        assert_eq!(
            opened(r#"{"v":1,"ceiling":"ten"}"#).ceiling(),
            Ceiling::default()
        );
    }

    #[test]
    fn a_number_is_the_ceiling() {
        assert_ne!(
            opened(r#"{"v":1,"ceiling":12.5}"#).ceiling(),
            Ceiling::default()
        );
    }
}
