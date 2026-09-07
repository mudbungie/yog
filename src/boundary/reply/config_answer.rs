//! **What a §9 config read answers**, as one type (bl-dd88) — the carrier that
//! matches the questions' own ([`config::Read`](crate::boundary::config::Read)).
//!
//! The five reads folded onto one carrier in bl-719a and their five answers did
//! not, so the roster named the family once on the asking side and five times
//! on the answering one. [`encode`](super::encode) had already collapsed them
//! into a single arm on the argument that *"the §9 config family's answers, one
//! arm since bl-2410: the carrier its questions were folded onto has a matching
//! set of replies, and the roster names the family once on this side too"* —
//! this is that sentence made true of the type as well.
//!
//! **The fold is in the carrier, never in the surface.** Each member keeps its
//! own reply `kind` on the wire, so no protocol version moves and the corpus
//! regenerates byte-identical; that identity is the check that the move was
//! behaviour-preserving, exactly as it was for the questions.

use super::ConfigView;

/// One §9 config read's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigAnswer {
    /// One §9 destination, raw bytes and typed settings both — see
    /// [`ConfigView`].
    File(ConfigView),
    /// brazen's effective provider table with the §5.1 #22 credential
    /// presence (§8.5, bl-0164) — the §8.3 login pane's own rows.
    Providers(Vec<crate::config_edit::brazen::ProviderRowView>),
    /// This workspace's role assignments (§9.4, §5.1 #27): the whole `roles:`
    /// block of the commit its lineage stands at, one row per role, in file
    /// order. The type is the grammar's own
    /// [`RoleModel`](crate::model_pick::RoleModel), which the §9.4 gestures
    /// already write and the fork composer already reads — one vocabulary for
    /// one entry (bl-2410).
    Roles(Vec<crate::model_pick::RoleModel>),
    /// The workspace's config lineages with each tip's files (§9.3, bl-dff8) —
    /// the config pane's two dropdowns.
    Lineages(Vec<crate::config_edit::branch::Lineage>),
    /// The model ids one provider offers (§9.4, bl-dff8) — the picker's roster.
    /// Never empty: a provider that offered nothing is a refusal saying so, not
    /// a list a seat would read as "no models exist".
    Models(Vec<String>),
    /// **The staged proposals** (§9.6, bl-dd88) — the listing, and one proposal
    /// whole when the read named one. See
    /// [`ProposalView`](crate::proposals::ProposalView).
    Proposals(crate::proposals::ProposalView),
}
