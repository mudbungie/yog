//! The §11 role stripe — who a message-bearing row speaks for, said once, at
//! the row's left edge, in one hue per role.
//!
//! The operator's ask (bl-3acb, verbatim): *"It needs to be easier to visually
//! discern user input and llm response, and other forms of inbox item."* The
//! answer is ONE mechanism applied uniformly — a thin vertical stripe on the
//! row's left edge — never a different trick per kind, and one mapping worn by
//! both seats that show a message: the transcript's rows and the
//! inbox-composer's pending queue. A message from a given author looks the
//! same pending as it does delivered, because both seats read this file and
//! neither restates a hue or a derivation.
//!
//! **The role is what the committed bytes say, never what the content reads
//! like.** In `.md` sender space the token `user` is reserved for the operator
//! (ARCH §2.11), an `epitaph:` marks a result deposit (§2.6), and any other
//! sender is a peer; model output is the `.json` model-id origin and the live
//! tail. [`message_role`] is the one spelling of that derivation, over each
//! seat's own sender authority (the filename token in the transcript, the
//! frontmatter `from:` in the queue — one deposit format, so they agree).
//!
//! **No hue is minted** (§11 single colour authority): gate violet is yog's
//! own — the operator's hand at the gate; spectral blue already means "a model
//! call producing text" (the live tail, the `InFlight` badge); brazen bronze is
//! the hue yog wears for *another agent's* doing (pending mail, the subagent
//! badge); and its tarnish is already "finished for now" (`Quiescent`) — a
//! child whose result deposit is its ending. The stripe paints no glyph, so
//! per the §11 glyph doctrine the words ride the mapping ([`role_badge`], the
//! `doing_badge` pair shape) and every stripe hovers them.

use super::{BRAZEN, BRAZEN_DIM, GATE, SPECTRE};

/// The reserved `.md` sender token naming the operator (ARCH §2.11: "In `.md`
/// sender space `user` stays reserved").
const USER_SENDER: &str = "user";

/// The stripe's width in points — enough hue to read at a glance, thin enough
/// to stay an edge and not a column.
const STRIPE_WIDTH: f32 = 3.0;

/// Who a message-bearing row speaks for — the closed §11 role vocabulary,
/// derived from committed bytes only (the sender token, the epitaph field, the
/// `.json` origin class). Machinery rows (thinking, tool calls, tool results,
/// raw bytes, turn rollups) have no role: nobody is speaking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The operator's own words: the reserved `user` sender.
    User,
    /// The agent speaking: model output, or the live streaming tail.
    Model,
    /// Any other sender — a peer agent's message into this inbox.
    Peer,
    /// A result deposit (`epitaph:` present, §2.6): a dispatched child's
    /// ending, arriving as mail rather than being chosen speech.
    Ended,
}

/// The role of a delivered/pending message, from the two facts its bytes
/// assert: who sent it and whether it carries an `epitaph:`. The epitaph wins
/// — a result deposit is a kind before it is a sender — then the reserved
/// `user` token, then the peer catch-all (the general path, so an absent or
/// unknown sender reads as third-party mail, never as the operator).
pub fn message_role(sender: &str, has_epitaph: bool) -> Role {
    if has_epitaph {
        Role::Ended
    } else if sender == USER_SENDER {
        Role::User
    } else {
        Role::Peer
    }
}

/// Hue + **the role said in words** — the badge-seat pair ([`super::badges`]'s
/// `doing_badge` shape: the stripe paints no glyph, so the words back the hue
/// on hover). Total over the enum, so a new role cannot ship wordless.
pub fn role_badge(role: Role) -> (egui::Color32, &'static str) {
    match role {
        Role::User => (GATE, "you — the operator's own message (sender: user)"),
        Role::Model => (SPECTRE, "the agent speaking — its model's own output"),
        Role::Peer => (BRAZEN, "another agent's message — third-party mail"),
        Role::Ended => (
            BRAZEN_DIM,
            "a result deposit — how a dispatched child ended",
        ),
    }
}

/// Paint the role stripe seat at the row's left edge: the role's hue with its
/// words on hover, or a blank spacer of the same width when nobody is speaking
/// — every row allocates the seat, so the toggles beside it stay aligned.
pub fn role_stripe(ui: &mut egui::Ui, role: Option<Role>) {
    let height = ui.text_style_height(&egui::TextStyle::Body);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(STRIPE_WIDTH, height), egui::Sense::hover());
    let Some(role) = role else {
        return;
    };
    let (hue, words) = role_badge(role);
    ui.painter().rect_filled(rect, 0.0, hue);
    response.on_hover_text(words);
}

#[cfg(test)]
mod tests;
