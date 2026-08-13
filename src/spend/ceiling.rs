//! The spend ceiling (DESIGN §3.5; VISION spend attribution): the spawn gate's **policy**
//! half — the operator's number and the comparison. The *seat* that enforces
//! it is [`crate::boundary::ceiling`], the one chokepoint every spawn crosses.
//!
//! The ruling this implements: the ceiling **gates spawns and never kills a
//! running drone**, because killing mid-ball destroys uncommitted work and
//! early termination is the expensive failure. So the only thing it can ever
//! refuse is a *birth* — nothing already running is touched, slowed or
//! stopped, and the bound on a drone that is already alive is lernie's own
//! `max_total_tokens`, one layer down, where the loop that spends it lives.
//!
//! **Severable in the strong sense, and in two directions.** The ceiling is
//! one `ui.json` number beside the price table (§4.1 `ceiling`); deleting the
//! key deletes the gate, not a code path — [`Ceiling::refusal`] is a `None`
//! away from an ungated yog. Deleting `prices` deletes it too, and that is not
//! an accident: a ceiling is a dollar figure, and yog refuses to bound dollars
//! it cannot compute rather than inventing a proxy.
//!
//! **The figure it compares is the workspace's** (§3.5's accepted
//! workspace-granularity attribution): a workspace is the sphere a drone lives
//! its ball in, and it is the one scope every spawn names outright without
//! inventing a linkage fact nobody stores.

use std::path::Path;

use serde_json::Value;

use super::{Cost, Prices};

/// The operator's spend ceiling, in micro-USD. `None` — the absent key — is
/// **no gate at all**, which is the default and the severability §3.5 demands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ceiling {
    limit: Option<u64>,
}

impl Ceiling {
    /// Read the `ui.json` value (§4.1 `ceiling`): a quoted USD number. Absent,
    /// non-numeric or negative all read as *no ceiling* — the forgiving read
    /// every `ui.json` key gets, so a typo costs the gate and never the window.
    /// A literal `0` is honored as written: the deliberate hard stop.
    pub fn from_json(value: Option<&Value>) -> Self {
        Self {
            limit: super::prices::quoted(value),
        }
    }

    /// The refusal a spawn into `workspace` earns, or `None` to let it fly.
    ///
    /// Three ways to fly: no ceiling configured, no price table (an unpriceable
    /// figure bounds nothing), or a workspace whose priced spend is still under
    /// the number. The comparison is against the figure's **floor** — tokens
    /// the table cannot price are reported by the §11 render and never guessed
    /// at here, so the gate refuses only on spend it can actually name.
    pub fn refusal(&self, workspace: &Path, prices: &Prices) -> Option<String> {
        self.verdict(&super::of_workspace(workspace, prices))
    }

    /// The same judgement over a figure someone else already folded — the
    /// **rendering** half (bl-66fb): the V4 board says where the ceiling will
    /// bind on the next spawn, and it must say it with the gate's own words and
    /// the gate's own comparison rather than a second opinion that could drift.
    /// The gate above is this function with the walk in front of it.
    pub fn verdict(&self, figure: &super::Figure) -> Option<String> {
        let limit = self.limit?;
        let cost = figure.cost?;
        (cost.micro_usd >= limit).then(|| {
            let ceiling = Cost {
                micro_usd: limit,
                unpriced_tokens: 0,
            };
            format!(
                "spend ceiling reached: this workspace has spent {} against a {} ceiling \
                 (ui.json `ceiling`), so nothing new is started here. Everything already \
                 running is untouched — raise the ceiling or delete the key to spawn again.",
                cost.usd(),
                ceiling.usd(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Ceiling;
    use crate::spend::Prices;
    use serde_json::json;
    use std::path::Path;

    const CONV: &str = "20260803T120000Z-root";

    /// $1/Mtok in — so a million input tokens is exactly $1.
    fn table() -> Prices {
        Prices::from_json(&json!({ "opus": { "input": 1 } }))
    }

    /// A workspace whose one step spent `input` tokens on the priced model.
    fn spent(dir: &Path, input: u64) {
        let step = dir.join("steps").join(CONV).join("001");
        std::fs::create_dir_all(&step).unwrap();
        std::fs::write(
            step.join("response.json"),
            format!(r#"{{"type":"usage","input_tokens":{input}}}"#),
        )
        .unwrap();
        std::fs::write(step.join("request.json"), r#"{"model":"opus"}"#).unwrap();
    }

    #[test]
    fn an_absent_or_malformed_key_is_no_gate() {
        for value in [None, Some(json!("lots")), Some(json!(-1))] {
            let ceiling = Ceiling::from_json(value.as_ref());
            assert_eq!(ceiling, Ceiling::default());
            let dir = tempfile::tempdir().unwrap();
            spent(dir.path(), 9_000_000);
            assert!(ceiling.refusal(dir.path(), &table()).is_none());
        }
    }

    #[test]
    fn an_unpriced_world_gates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        spent(dir.path(), 9_000_000);
        let ceiling = Ceiling::from_json(Some(&json!(1)));
        assert!(ceiling.refusal(dir.path(), &Prices::default()).is_none());
    }

    #[test]
    fn under_the_ceiling_flies() {
        let dir = tempfile::tempdir().unwrap();
        spent(dir.path(), 2_000_000);
        let ceiling = Ceiling::from_json(Some(&json!(2.5)));
        assert!(ceiling.refusal(dir.path(), &table()).is_none());
    }

    #[test]
    fn at_the_ceiling_refuses_and_names_both_figures() {
        let dir = tempfile::tempdir().unwrap();
        spent(dir.path(), 3_000_000);
        let refusal = Ceiling::from_json(Some(&json!(2.5)))
            .refusal(dir.path(), &table())
            .unwrap();
        assert!(refusal.contains("$3.00"), "{refusal}");
        assert!(refusal.contains("$2.50"), "{refusal}");
        assert!(refusal.contains("untouched"), "{refusal}");
    }

    #[test]
    fn zero_is_the_hard_stop() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            Ceiling::from_json(Some(&json!(0)))
                .refusal(dir.path(), &table())
                .is_some(),
            "a ceiling of 0 refuses a spawn into an unspent workspace"
        );
    }
}
