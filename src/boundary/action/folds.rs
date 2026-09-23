//! **Why six families ride ONE variant over their own `Verb`** (bl-d255):
//! prose only — nothing in this file is read by a program. The split is
//! [`conversation`](super::conversation)'s, and this module is the one place
//! the fold rule is stated: the roster said it six times, once per family, and
//! six statements of one fact are six chances to drift (§12's own criterion).
//!
//! **The rule.** An enum cannot be split across files, so the roster makes
//! room the only way a roster can: a family whose members every layer beneath
//! already reads as a set folds to one variant over that family's own `Verb`.
//! It is a real seam and not a line budget precisely because the seam is
//! already drawn three files down — `codec::<family>` spells the members,
//! `line::<family>` reads them, `boundary::<family>` executes them,
//! `reply::<family>` answers them — and each carrier's own doc says what its
//! members are and what each spends. **No wire spelling moves when a family
//! folds**: every member keeps its own op word, and the check is that
//! `corpus/` regenerates byte-identical.
//!
//! **[`Ball`](super::Action::Ball)** (§8.2, bl-92d3) — close, claim, unclaim,
//! create, update over [`verbs::Verb`](crate::actions::verbs::Verb). bl-92d3 is
//! also the rule about the wall itself: the roster came to rest at 299 against
//! a 300 cap, which inverts the cap, and the fold was taken *before* the next
//! boundary act rather than during one.
//!
//! **[`Fan`](super::Action::Fan)** (§3.8, VISION §4.10, bl-8746) — spread one
//! delivery obligation into N isolated candidates, or retire one of them, over
//! [`fan::Verb`](crate::fan::Verb).
//!
//! **[`Monitor`](super::Action::Monitor)** (VISION §4.9, rung V6) — arm a
//! workspace on a cheap model, disarm it, or raise an attention item on one
//! conversation, over [`monitor::Verb`](crate::monitor::Verb): same subject,
//! same config file, same trail. Arming is the operator's explicit action and
//! it *is* the mechanism: unarmed, no call is made, no row is written and
//! nothing renders.
//!
//! **[`Fleet`](super::Action::Fleet)** (VISION §4.3, rung V4 item 2) — arm one
//! workspace's fleet loop on a project and a cap, or disarm it, over
//! [`fleet::Verb`](crate::fleet::Verb). **Arming is the explicit user action
//! and it *is* the mechanism** (I7, §4.3): unarmed nothing spawns, nothing is
//! reaped, nothing renders; an armed loop's spawns are that action, continuing.
//! Severability is deleting the `cadence.yaml` entry, never editing a code path.
//!
//! **[`Config`](super::Action::Config)** (§9, bl-dd88) — over
//! [`config::Write`](crate::boundary::config::Write), the carrier matching the
//! questions' own ([`config::Read`](crate::boundary::config::Read)): the apply,
//! the §16.3 marks knob, the §9.4 pick and its tuning pair, and the §9.6
//! proposal settle.
//!
//! **[`Route`](super::Action::Route)** (REMOTE §5, §9 step 7; bl-024b) — queue
//! one tool call for the machine that advertised it, and post back what running
//! it captured, over [`mailbox::Verb`](crate::registry::mailbox::Verb): one
//! subject, one mailbox, one pair of ends.
