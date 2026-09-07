//! **What a §9 config read answers** (bl-dd88) — the family's own fixtures,
//! cut off [`super::listings`] at §12's per-file budget on the seam the answer
//! carrier already draws: everything left there is a listing some pane asks
//! for, and these five are one family with one carrier
//! ([`ConfigAnswer`](crate::boundary::reply::ConfigAnswer)).
//!
//! Each member carries both readings of every field it has, because a fixture
//! set that only ever spells the easy case proves only that the easy case
//! survives.

use super::super::super::super::super::Reply;
use crate::boundary::reply::ConfigAnswer;
use crate::config_edit::brazen::ProviderRowView;

/// The §9 family's answers, in carrier order.
pub(super) fn answers() -> Vec<Reply> {
    vec![
        // The §9.4 role assignments (bl-2410): a role tuned both ways beside
        // one tuned neither, so `effort`'s two spellings and `priority`'s two
        // both cross. A level yog would never WRITE rides here too, because
        // this answer reports the file rather than asserting a vocabulary.
        Reply::Config(ConfigAnswer::Roles(vec![
            crate::model_pick::RoleModel {
                role: "worker".into(),
                provider: "anthropic".into(),
                model: "claude-sonnet-5".into(),
                effort: Some("high".into()),
                priority: true,
            },
            crate::model_pick::RoleModel {
                role: "compactor".into(),
                provider: "codex".into(),
                model: "gpt-5.4-mini".into(),
                effort: None,
                priority: false,
            },
            crate::model_pick::RoleModel {
                role: "critic".into(),
                provider: "codex".into(),
                model: "gpt-5.4".into(),
                effort: Some("extreme".into()),
                priority: false,
            },
        ])),
        // The §9.6 staged proposals (bl-dd88): one fresh row beside one stale,
        // so both readings of `fresh` cross and the empty lineage pool the
        // stale one wears is proven rather than assumed; and the listing with
        // one named whole beside the bare one, so `whole`'s presence and its
        // absence are both fixtures.
        Reply::Config(ConfigAnswer::Proposals(crate::proposals::ProposalView {
            rows: vec![
                crate::proposals::ProposalRow {
                    id: "20260906T090000Z-r001".into(),
                    lineages: vec!["default".into()],
                    parent: "9f2c1ab4".into(),
                    fresh: true,
                    diffstat: "1 file changed, 6 insertions(+)".into(),
                    subject: "notes: record what the span taught".into(),
                },
                crate::proposals::ProposalRow {
                    id: "20260906T091500Z-r002".into(),
                    lineages: vec![],
                    parent: "3ac70e11".into(),
                    fresh: false,
                    diffstat: "2 files changed, 9 insertions(+), 1 deletion(-)".into(),
                    subject: "skills: the apk cache is worth keeping".into(),
                },
            ],
            whole: None,
        })),
        Reply::Config(ConfigAnswer::Proposals(crate::proposals::ProposalView {
            rows: vec![crate::proposals::ProposalRow {
                id: "20260906T090000Z-r001".into(),
                lineages: vec!["default".into()],
                parent: "9f2c1ab4".into(),
                fresh: true,
                diffstat: "1 file changed, 6 insertions(+)".into(),
                subject: "notes: record what the span taught".into(),
            }],
            whole: Some("commit 71011c3d\n\n    notes: …\n\n+ a line\n".into()),
        })),
        Reply::Config(ConfigAnswer::Providers(vec![
            ProviderRowView {
                name: "anthropic".into(),
                fact: "credential present".into(),
                blocked: None,
                effort: true,
                priority: true,
            },
            // The other arm of both tuning booleans, so neither is only ever
            // spelled one way across the surface (bl-23bd).
            ProviderRowView {
                name: "openai".into(),
                fact: "no credential".into(),
                blocked: Some("no login flow".into()),
                effort: false,
                priority: false,
            },
        ])),
    ]
}
