//! The §3.5 spend ceiling's one home in `ui.json` (DESIGN §4.1 `ceiling`) —
//! beside the price table ([`super::prices`]) it is denominated in, and read
//! and written exactly like it: through the boundary since bl-53d1
//! (`Action::Ceiling`), by the same write-through, for the reasons that file
//! states — the hand edit is still live within a tick through the whole-file
//! `adopt` (§4.1 I5), and the seat that would make it is another machine.
//!
//! Absent ⇒ no ceiling ⇒ no gate. That is the severability the ruling demands
//! and it is one `get` away from being obvious.

use super::UiState;
use crate::spend::Ceiling;

/// The ceiling key (§4.1): the operator's spend bound in USD.
const CEILING: &str = "ceiling";

impl UiState {
    /// The §3.5 spend ceiling. Absent, or of the wrong shape, reads as *no
    /// ceiling* — deleting the key deletes the gate, not a code path.
    pub fn ceiling(&self) -> Ceiling {
        Ceiling::from_json(self.world.root.get(CEILING))
    }

    /// Write the number, in micro-USD, or delete the key (`None`) —
    /// `Action::Ceiling`'s write-through (bl-53d1). Stored as the USD decimal
    /// the operator quoted, which is the shape the read above takes.
    pub fn set_ceiling(&mut self, micro_usd: Option<u64>) {
        match micro_usd {
            Some(micro) => {
                self.world
                    .root
                    .insert(CEILING.to_owned(), crate::spend::decimal(micro));
            }
            None => {
                self.world.root.remove(CEILING);
            }
        }
        self.world.save();
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

    /// The write-through, both directions: a number set reads back as that
    /// ceiling from the file, and `None` deletes the key.
    #[test]
    fn a_ceiling_written_is_read_back_and_none_deletes_the_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ui.json");
        let mut ui = UiState::open(path.clone());
        ui.set_ceiling(Some(12_500_000));
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("\"ceiling\": 12.5")
        );
        assert_eq!(
            UiState::open(path.clone()).ceiling().micro_usd(),
            Some(12_500_000)
        );
        ui.set_ceiling(None);
        assert_eq!(UiState::open(path).ceiling(), Ceiling::default());
    }

    #[test]
    fn a_number_is_the_ceiling() {
        assert_ne!(
            opened(r#"{"v":1,"ceiling":12.5}"#).ceiling(),
            Ceiling::default()
        );
    }
}
