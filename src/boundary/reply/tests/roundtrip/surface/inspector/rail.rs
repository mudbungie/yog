//! The step spine's fixture (VISION V1): a pinnable notch and an unreachable
//! one, a card mid-sentence and a silent one — the four absences the encoder
//! spells as absent keys — and, since bl-53d1, both arms of the priced rollup.

use crate::git_tree::AgentState;
use crate::rail::{ChildCard, Notch, Place, Rail};

/// A pinnable notch and an unreachable one; a card mid-sentence and a silent
/// one — the four absences the encoder spells as absent keys.
pub(super) fn rail() -> Rail {
    Rail {
        notches: vec![
            Notch {
                seq: "001".into(),
                commit: Some("abcdef1234567890".into()),
                budget: 120,
                // The rollup priced (bl-53d1) on the pinnable notch, and the
                // absent arm on the unreachable one.
                cost: Some(crate::spend::Cost {
                    micro_usd: 1_200_000,
                    unpriced_tokens: 20,
                }),
                place: Some(Place {
                    row: "003-claude.json".into(),
                    cut: 2,
                }),
            },
            Notch {
                seq: "002".into(),
                commit: None,
                budget: 120,
                cost: None,
                place: None,
            },
        ],
        cards: vec![
            ChildCard {
                agent_id: "c-1-a".into(),
                name: "Cobalt".into(),
                fork: "from here".into(),
                state: AgentState::Live,
                tokens: 9,
                tail: Some("working".into()),
                provenance_notch: 0,
            },
            ChildCard {
                agent_id: "c-1-b".into(),
                name: "Dun".into(),
                fork: "from config/main".into(),
                state: AgentState::Stopped,
                tokens: 0,
                tail: None,
                provenance_notch: 1,
            },
        ],
    }
}
