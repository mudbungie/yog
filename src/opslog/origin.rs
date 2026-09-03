//! Which surface an attempted action came from (DESIGN §7.3, §11) — the one
//! fact the ops row lacked, and without which "the originating surface renders
//! the failure" was unhonorable.
//!
//! A row's exit and its stderr say what went wrong and nothing about *whose*
//! failure it is, so every banner surface asked the same global question ("did
//! the last op fail?") and a single start failure painted itself on all of them
//! at once (bl-48f8). The attribution is a fact of the **gesture**,
//! not of the row's bytes: `bl close` and `litany message` are told apart by
//! their argv, but `litany prompt` / `litany new` / `["yog-step","mkdir"]` are
//! written identically by a ball-rung start and by the composer's own Enter. So
//! origin is recorded once, where the op fires and the rung is known, and every
//! surface reads it back rather than re-deriving it from a shape that cannot say.
//!
//! It is the op's **subject**, not the pixel the pointer was over: one gesture
//! has one body however many hands reach it (`ball_bar::close_ball` is the
//! composer's button, the `c` key and the §11 row menu), and forking that body
//! per hand to record a pointer position would buy a distinction no operator
//! makes. A ball verb is about a ball wherever it was clicked.

/// The subject an attempted action acted on — the §7.3 attribution a banner
/// surface filters by, and the field it groups the answered rows standing
/// [`Live`](super::Standing::Live) under ([`super::standings`]).
///
/// Three, because yog has three kinds of subject and one banner surface each:
/// the roster's balls section paints [`Balls`](Self::Balls), the composer (the
/// empty world's bootstrap box being the same box with no workspace yet) paints
/// [`Conversation`](Self::Conversation), and [`World`](Self::World) is what the
/// config editors, the marks knob, the login pane and the delete dialog render
/// *themselves* — those surfaces already say their own errors, and routing them
/// through a banner on an unrelated surface is the bug this enum closes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Origin {
    /// A **ball** op: every `bl` verb, and every step of a ball-rung start —
    /// including its substrate steps, which name no ball but serve one. Rendered
    /// by the roster's balls section, where the ▶ Start row that fired it is
    /// (§11, bl-6ad8).
    Balls,
    /// A **conversation** op: `litany message`/`stop`/`scan`, and every step of a
    /// bare- or path-rung start. Rendered by the composer.
    ///
    /// The default, so a line written by an older yog — or by any writer that
    /// forgot to say — still banners exactly once instead of vanishing (INV-2:
    /// no error is printed and dropped). The composer is the one surface that is
    /// always on screen in some form, which is what makes it the safe fallback.
    #[default]
    Conversation,
    /// A **world** op: the §9 config writes, the §16.3 marks knob, the §8.3
    /// login flow, the §3.6 unmaking, yog's own §7.2 drift observations, and
    /// the operator's own two trail gestures — the ack and the clear
    /// ([`super::operator`], §4.2), which are *about* the banners and so must
    /// never raise one.
    /// No §7.3 banner renders these — each of those surfaces states its own
    /// outcome in place, and a config-write failure is not news the composer has
    /// any business breaking.
    World,
}

/// The token a line's `origin` field carries (§4.2) — short, stable, and the
/// only spelling; [`Origin::parse`] is its exact inverse.
const BALLS: &str = "balls";
const CONVERSATION: &str = "conversation";
const WORLD: &str = "world";

impl Origin {
    /// This origin's durable token. `pub(crate)`: the token is the §4.2 line
    /// codec's business and nobody else's — outside this crate an origin is the
    /// enum, never its spelling.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Balls => BALLS,
            Self::Conversation => CONVERSATION,
            Self::World => WORLD,
        }
    }

    /// The origin a line's token names. Anything else — an absent field, a
    /// token from a future yog — is the [`Conversation`](Self::Conversation)
    /// default: forgiving, like every other field of the §4.2 parser.
    /// `pub(crate)` for the same reason as [`as_str`](Self::as_str).
    pub(crate) fn parse(token: &str) -> Self {
        match token {
            BALLS => Self::Balls,
            WORLD => Self::World,
            _ => Self::Conversation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Origin;

    #[test]
    fn every_token_round_trips() {
        for origin in [Origin::Balls, Origin::Conversation, Origin::World] {
            assert_eq!(Origin::parse(origin.as_str()), origin);
        }
    }

    /// A line an older yog wrote carries no token at all, and one a future yog
    /// wrote may carry a word this build has never heard of. Both read as the
    /// composer's — so the failure still banners once (INV-2), never nowhere.
    #[test]
    fn an_unknown_or_absent_token_falls_back_to_the_composer() {
        assert_eq!(Origin::parse(""), Origin::Conversation);
        assert_eq!(Origin::parse("inspector"), Origin::Conversation);
        assert_eq!(Origin::default(), Origin::Conversation);
    }
}
