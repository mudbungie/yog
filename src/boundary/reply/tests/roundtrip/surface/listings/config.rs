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
use crate::boundary::reply::{ConfigAnswer, Place, Placed};
use crate::config_edit::brazen::ProviderRowView;

/// A row on the registered machine `client`, spelled `row` there (REMOTE
/// §5.7, bl-471d) — the two keys only such a row carries.
fn on<T>(client: &str, row: &str, listed: T) -> Placed<T> {
    let (client, row) = (client.to_owned(), row.to_owned());
    Placed {
        row: listed,
        at: Some(Place { client, row }),
    }
}

/// The §9 family's answers, in carrier order.
pub(super) fn answers() -> Vec<Reply> {
    vec![
        roles(),
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
            }
            .into(),
            // The other arm of both tuning booleans, so neither is only ever
            // spelled one way across the surface (bl-23bd).
            ProviderRowView {
                name: "openai".into(),
                fact: "no credential".into(),
                blocked: Some("no login flow".into()),
                effort: false,
                priority: false,
            }
            .into(),
            // A serving machine's row, whose `client` and `row` say what the
            // name is made of — and that a client's name may carry the joiner.
            on("lab@annex", "claude-sub", far("claude-sub@lab@annex")),
        ])),
        // The sign-in catalog (bl-c6e3): the same row shape under its own
        // kind, carrying the one row the table above cannot — an oauth preset
        // nobody has signed into yet, whose Login verb is exactly the point.
        Reply::Config(ConfigAnswer::Catalog(vec![
            ProviderRowView {
                name: "anthropic".into(),
                fact: "auth oauth2 \u{b7} signed in".into(),
                blocked: None,
                effort: true,
                priority: true,
            }
            .into(),
            ProviderRowView {
                name: "openai-chatgpt".into(),
                fact: "auth oauth2 \u{b7} not signed in".into(),
                blocked: None,
                effort: false,
                priority: false,
            }
            .into(),
            on("laptop", "claude-sub", far("claude-sub@laptop")),
        ])),
    ]
}

/// A registered machine's oauth row, signed in there.
fn far(name: &str) -> ProviderRowView {
    ProviderRowView {
        name: name.into(),
        fact: "auth oauth2 \u{b7} signed in".into(),
        blocked: None,
        effort: true,
        priority: false,
    }
}

/// The §9.4 role assignments (bl-2410): a role tuned both ways beside
/// one tuned neither, so `effort`'s two spellings and `priority`'s two
/// both cross. A level yog would never WRITE rides here too, because
/// this answer reports the file rather than asserting a vocabulary.
fn roles() -> Reply {
    Reply::Config(ConfigAnswer::Roles(vec![
        crate::model_pick::RoleModel {
            role: "worker".into(),
            provider: "anthropic".into(),
            model: "claude-sonnet-5".into(),
            effort: Some("high".into()),
            priority: true,
        }
        .into(),
        crate::model_pick::RoleModel {
            role: "compactor".into(),
            provider: "codex".into(),
            model: "gpt-5.4-mini".into(),
            effort: None,
            priority: false,
        }
        .into(),
        // A role on a registered machine's row: the name whole, and its
        // two halves beside it.
        on(
            "laptop",
            "codex",
            crate::model_pick::RoleModel {
                role: "critic".into(),
                provider: "codex@laptop".into(),
                model: "gpt-5.4".into(),
                effort: Some("extreme".into()),
                priority: false,
            },
        ),
    ]))
}
