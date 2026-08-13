//! The **flat receipts** (§8.5): the replies that carry no rows at all — a
//! kind, and at most a flag or two saying which way a write went. Their own
//! file at §12's cap (bl-94b4), on the seam every sibling here is cut along:
//! everything else in this directory encodes a *list* or a derived sub-object,
//! and these encode a fact about the gesture that just ran.

use super::*;

#[test]
fn every_flat_receipt_says_what_happened() {
    let started = encode(&Reply::Started {
        conversation: "brave-fox".into(),
    });
    assert_eq!(started["ok"], true);
    assert_eq!(started["conversation"], "brave-fox");
    let deleted = encode(&Reply::Deleted);
    assert_eq!(deleted["kind"], "deleted");
    assert_eq!(deleted["ok"], true);
    assert_eq!(encode(&Reply::Acked)["kind"], "acked");
    assert_eq!(encode(&Reply::TrailCleared)["kind"], "trail-cleared");
    // The VISION §4.9 monitor's two: arming says which way it went, so a seat
    // never has to re-read the config file to know what it just did.
    let armed = encode(&Reply::Armed { armed: true });
    assert_eq!(armed["kind"], "armed");
    assert_eq!(armed["armed"], true);
    assert_eq!(encode(&Reply::Armed { armed: false })["armed"], false);
    assert_eq!(encode(&Reply::Flagged)["kind"], "flagged");
    // The §8.6 answer names the call it landed on — the receipt an audit reads
    // — and says whether the release was actually launched.
    let answered = encode(&Reply::Answered {
        tool_use: "toolu_42".into(),
        tool: "bash".into(),
        ruling: crate::control::judge::Ruling::Pass,
        advanced: true,
    });
    assert_eq!(answered["kind"], "answered");
    assert_eq!(answered["tool_use"], "toolu_42");
    assert_eq!(answered["tool"], "bash");
    assert_eq!(answered["verdict"], "pass");
    assert_eq!(answered["advanced"], true);
    // The §4.9 fifth rung's receipt states what **stands**, not what was asked
    // — so a restore under an ancestor's floor cannot read as a restore.
    let floored = encode(&Reply::Floored { standing: true });
    assert_eq!(floored["kind"], "floored");
    assert_eq!(floored["standing"], true);
    assert_eq!(
        encode(&Reply::Floored { standing: false })["standing"],
        false
    );
}
