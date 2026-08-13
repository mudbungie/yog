//! The three-valued answer (VISION §4.9) and the one reading of a model's
//! reply.
//!
//! The check's whole response is `aligned | drifting | diverged` plus one
//! sentence. Three values and no fourth: a call that failed, timed out or came
//! back unreadable is **not** a verdict — it leaves the last-checked sha behind
//! the branch tip and the next tick re-fires (the anti-reinvention law's retry).
//! So [`read`] returns `None` rather than inventing a degraded class, and the
//! monitor never stores an "unknown" that a surface would have to render.

/// One verdict. Ordered least to most divergent, which is also the order a
/// standing verdict escalates in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    Aligned,
    Drifting,
    Diverged,
}

/// The durable tokens — the ops row's spelling and the prompt's vocabulary in
/// one place, so the policy file and the parser cannot drift apart.
const ALIGNED: &str = "aligned";
const DRIFTING: &str = "drifting";
const DIVERGED: &str = "diverged";

impl Verdict {
    /// This verdict's token: what the ops row carries and what the model is
    /// asked to say. `pub(crate)` for the same reason
    /// [`Origin::as_str`](crate::opslog::Origin) is — the token is the durable
    /// codec's business; outside this crate a verdict is the enum, never its
    /// spelling.
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Aligned => ALIGNED,
            Self::Drifting => DRIFTING,
            Self::Diverged => DIVERGED,
        }
    }

    /// The verdict a token names, or `None` for anything else. Exact and
    /// case-insensitive: the model is told to answer with one of three words,
    /// and a reply that does not is a failed check, not a near miss to round.
    pub fn parse(token: &str) -> Option<Self> {
        match token.trim().to_ascii_lowercase().as_str() {
            ALIGNED => Some(Self::Aligned),
            DRIFTING => Some(Self::Drifting),
            DIVERGED => Some(Self::Diverged),
            _ => None,
        }
    }
}

/// A read reply: the verdict and the one sentence behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub verdict: Verdict,
    pub reason: String,
}

/// Read a model's answer. The shape asked for is `<verdict>: <one sentence>`;
/// the leading token is taken up to the first `:` (or the first whitespace when
/// the model omitted the colon), and the rest of the *first* line is the reason.
/// Everything after that first line is dropped: the check asked for one
/// sentence, and honoring more of the reply than was asked for is how a bounded
/// call starts becoming a conversation.
pub fn read(text: &str) -> Option<Reply> {
    let line = text.lines().map(str::trim).find(|l| !l.is_empty())?;
    let (head, rest) = match line.split_once(':') {
        Some(split) => split,
        None => line.split_once(char::is_whitespace).unwrap_or((line, "")),
    };
    Some(Reply {
        verdict: Verdict::parse(head)?,
        reason: rest.trim().trim_start_matches(['-', '—']).trim().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_token_round_trips() {
        for verdict in [Verdict::Aligned, Verdict::Drifting, Verdict::Diverged] {
            assert_eq!(Verdict::parse(verdict.token()), Some(verdict));
        }
        assert_eq!(Verdict::parse("ALIGNED "), Some(Verdict::Aligned));
        assert_eq!(Verdict::parse("mostly aligned"), None);
    }

    /// The order is the escalation order, which is what makes "the worst of a
    /// subtree" a `max` rather than a table.
    #[test]
    fn the_verdicts_order_by_divergence() {
        assert!(Verdict::Aligned < Verdict::Drifting && Verdict::Drifting < Verdict::Diverged);
    }

    #[test]
    fn the_asked_for_shape_reads() {
        assert_eq!(
            read("diverged: it is refactoring an unrelated crate\n"),
            Some(Reply {
                verdict: Verdict::Diverged,
                reason: "it is refactoring an unrelated crate".to_owned(),
            })
        );
    }

    #[test]
    fn a_missing_colon_a_dash_and_leading_blanks_still_read() {
        assert_eq!(
            read("\n\n  aligned — writing the tests it was asked for  \nand more\n").expect("read"),
            Reply {
                verdict: Verdict::Aligned,
                reason: "writing the tests it was asked for".to_owned(),
            },
            "the first non-empty line is the whole answer"
        );
        assert_eq!(
            read("drifting").expect("read").reason,
            "",
            "a bare verdict is a verdict with no sentence, not a failure"
        );
    }

    #[test]
    fn an_unreadable_reply_is_not_a_verdict() {
        assert_eq!(read(""), None);
        assert_eq!(read("I cannot judge this.\n"), None);
        assert_eq!(read("   \n"), None);
    }
}
