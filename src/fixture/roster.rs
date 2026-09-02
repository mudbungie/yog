//! **The named states**, and the whole of what a consumer may ask for. One
//! `const` per name, looked up by [`resolve`]; the names are the contract, so
//! adding one is additive and renaming one is a break.
//!
//! Every state is written as a departure from
//! [`Recipe::empty`](super::recipe::Recipe::empty) — DESIGN §3.4's own posture,
//! *the general path with empty inputs* — so the first-run state is not a
//! special case here any more than it is in the engine.

use super::recipe::{Conv, Recipe, Step, Wsp};

/// The one workspace name every non-empty state uses. DESIGN §3.1's bootstrap
/// constant — a fixture that invented its own would answer a name no
/// empty-world start ever produces.
pub const WORKSPACE: &str = crate::names::DEFAULT_NAME;

/// **first run**: a seeded world with no workspace at all.
const EMPTY: Recipe = Recipe::empty("a seeded world with no workspaces — the first-run state");

/// **a busy workspace**: six conversations covering every resting §3.5 state
/// and every `refs/litany/*` mark, dated apart so the §11 roster's sort is the
/// recipe's and not the clock's.
const BUSY: Recipe = Recipe {
    summary: "one workspace, six conversations across every resting state and mark",
    workspaces: &[Wsp {
        name: WORKSPACE,
        convs: &[
            Conv {
                age_secs: 60,
                step: Step::Streaming,
                messages: &[("001-op.md", "Draft the release note.\n")],
                ..Conv::new("c-001", "Ball bl-1001: draft the release note\n")
            },
            Conv {
                age_secs: 420,
                step: Step::Settled,
                marks: &["notify"],
                messages: &[
                    ("001-op.md", "Summarise the changelog.\n"),
                    ("002-model.json", MODEL_TURN),
                ],
                ..Conv::new("c-002", "Ball bl-1002: summarise the changelog\n")
            },
            Conv {
                age_secs: 900,
                step: Step::Failed,
                marks: &["conflicted"],
                ..Conv::new("c-003", "Ball bl-1003: rebase the delivery branch\n")
            },
            Conv {
                age_secs: 1_800,
                step: Step::OutputLimit,
                ..Conv::new("c-004", "Ball bl-1004: enumerate the module map\n")
            },
            Conv {
                age_secs: 3_600,
                step: Step::Settled,
                marks: &["budget-exhausted"],
                deposits: &[("001-op", "One more pass, please.\n")],
                ..Conv::new("c-005", "Ball bl-1005: widen the coverage floor\n")
            },
            Conv {
                age_secs: 7_200,
                marks: &["abandoned"],
                ..Conv::new("c-006", "Ball bl-1006: retire the paint walk\n")
            },
        ],
    }],
    cadence: None,
    brazen: None,
};

/// **the §7.3 wound**, both arms: one driver that left words behind and one
/// that left none. The banner a seat paints is gated by its own
/// `wound_grace` window — see [`super`] on why that gate is the seat's and
/// not this fixture's.
const WOUND: Recipe = Recipe {
    summary: "two wounded conversations: one whose stderr.log speaks, one mute",
    workspaces: &[Wsp {
        name: WORKSPACE,
        convs: &[
            Conv {
                age_secs: 120,
                step: Step::Wound(
                    "brazen: unknown provider `openai-chatgpt` in this workspace's wall\n",
                ),
                messages: &[
                    ("001-op.md", "Start the sweep.\n"),
                    ("002-model.json", MODEL_TURN),
                ],
                ..Conv::new("c-101", "Ball bl-1101: sweep the tree\n")
            },
            Conv {
                age_secs: 240,
                step: Step::Wound(""),
                messages: &[("001-op.md", "Carry on.\n"), ("002-model.json", MODEL_TURN)],
                ..Conv::new("c-102", "Ball bl-1102: carry the turn on\n")
            },
        ],
    }],
    cadence: None,
    brazen: None,
};

/// **the orphaned tail**, both shapes: mail nobody is answering, and a turn
/// that died inside its tool window.
const ORPHAN: Recipe = Recipe {
    summary: "an orphaned delivered message and an orphaned tool window",
    workspaces: &[Wsp {
        name: WORKSPACE,
        convs: &[
            Conv {
                age_secs: 180,
                step: Step::Settled,
                messages: &[("001-model.json", MODEL_TURN), ("002-op.md", "And now?\n")],
                driver_log: "litany: driver exited 137 (killed)\n",
                ..Conv::new("c-201", "Ball bl-1201: answer the deposit\n")
            },
            Conv {
                age_secs: 300,
                step: Step::Settled,
                messages: &[
                    ("001-op.md", "List the tree.\n"),
                    ("002-model.json", TOOL_USE),
                ],
                ..Conv::new("c-202", "Ball bl-1202: read the tree\n")
            },
        ],
    }],
    cadence: None,
    brazen: None,
};

/// **a long transcript**: entries either side of a compaction hole, one of
/// them far past every preview and input-summary cap.
const TRANSCRIPT: Recipe = Recipe {
    summary: "one conversation with a compacted transcript and an over-long entry",
    workspaces: &[Wsp {
        name: WORKSPACE,
        convs: &[Conv {
            age_secs: 90,
            step: Step::Settled,
            summaries: &[(
                "012.md",
                "The first eleven turns established the module map.\n",
            )],
            messages: &[
                ("013-op.md", LONG),
                ("014-model.json", MODEL_TURN),
                ("015-tool.json", TOOL_RESULT),
                ("016-model.json", LONG_TURN),
                ("017-model.json", MODEL_TURN),
            ],
            ..Conv::new("c-301", "Ball bl-1301: carry a long transcript\n")
        }],
    }],
    cadence: None,
    brazen: None,
};

/// **settings present**: a tuned cadence and a workspace wall carrying
/// provider rows — the two settings surfaces a seat renders controls over.
const SETTINGS: Recipe = Recipe {
    summary: "a tuned cadence.yaml and a workspace wall carrying provider rows",
    workspaces: &[Wsp {
        name: WORKSPACE,
        convs: &[Conv {
            age_secs: 300,
            step: Step::Settled,
            ..Conv::new("c-401", "Ball bl-1401: read the settings\n")
        }],
    }],
    cadence: Some("cadence:\n  debounce_ms: 250\n  cheap_sweep_ms: 2000\n"),
    brazen: Some("[providers.anthropic]\nkind = \"anthropic\"\n"),
};

/// Every name, with its recipe, in the order `yog fixture` lists them.
pub const ROSTER: &[(&str, &Recipe)] = &[
    ("empty", &EMPTY),
    ("busy", &BUSY),
    ("wound", &WOUND),
    ("orphan", &ORPHAN),
    ("transcript", &TRANSCRIPT),
    ("settings", &SETTINGS),
];

/// The recipe `name` asks for, or `None` — the caller owns the refusal, because
/// it is the one place that knows the whole roster is worth printing beside it.
pub fn resolve(name: &str) -> Option<&'static Recipe> {
    ROSTER
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, recipe)| *recipe)
}

/// Every state name, for a refusal and for the bare listing.
pub fn names() -> Vec<String> {
    ROSTER.iter().map(|(name, _)| (*name).to_owned()).collect()
}

/// An ordinary assistant turn.
const MODEL_TURN: &str =
    "{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"Done.\"}]}\n";

/// An assistant turn whose tool call nothing answers — the orphaned
/// tool-window tail.
const TOOL_USE: &str = "{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\
     \"id\":\"tu-1\",\"name\":\"Bash\",\"input\":{\"command\":\"ls\"}}]}\n";

/// The result that answers one.
const TOOL_RESULT: &str = "{\"role\":\"user\",\"content\":[{\"type\":\"tool_result\",\
     \"tool_use_id\":\"tu-1\",\"content\":\"AGENTS.md\\nsrc\\n\"}]}\n";

/// A deposit far past every preview cap, so a seat's elision is exercised
/// rather than assumed.
const LONG: &str = "Walk the whole module map and report every file that has crossed two \
     hundred lines, with the seam you would split it on, the DESIGN section that would \
     carry the new row, and whether the split is one a reader would recognise a week \
     later or only a line count would.\n";

/// The same length, as a model turn.
const LONG_TURN: &str = "{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\
     \"Eleven files are over the aspiration and none over the wall; the seams are real in \
     eight of them and a line count in three, which is the number worth arguing about.\"}]}\n";
