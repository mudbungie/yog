//! **Why the acts a REMOTE device makes are shaped the way they are**
//! (bl-d255): prose only — nothing in this file is read by a program. The
//! split is [`conversation`](super::conversation)'s.
//!
//! **[`Advertise`](super::Action::Advertise)** is a tool host presenting its
//! set (REMOTE §5, bl-4e08): the three facts per element — name, description,
//! JSON Schema verbatim — which the engine writes into that client's
//! registration when they differ from what is stored
//! ([`registry::tools`](crate::registry::tools)).
//!
//! **It names no client, and that is the gesture.** The identity it lands
//! under is the *intake's* — a connection's certificate common name, read
//! exactly where scoping reads it (REMOTE §4) — because a client field would
//! let any connection overwrite any other client's set, which is the
//! authorization the certificate already decided. An intake carrying no client
//! identity (the `gestures/` inbox, `yog gesture`, the window) therefore
//! refuses in band: it is a boundary verb like any other, and the wire gains
//! nothing it does not (REMOTE §3).
//!
//! **[`Enroll`](super::Action::Enroll)** enrolls a device (REMOTE §1.4 as
//! amended, §8.4; bl-f4e3) — one variant over
//! [`enroll::Request`](crate::registry::enroll::Request), whose own doc carries
//! the ruling and the §4.2 grade a foot is refused it by.
//!
//! **[`Login`](super::Action::Login)** is the sign-in, as an act (REMOTE §8.3;
//! DESIGN §8.3 as amended by bl-61bf): start `bz --login` on the ENGINE inside
//! the named workspace's wall, so the credential lands where that workspace's
//! agents read it whichever box the seat is on. It never waits for the run and
//! its receipt is that run's standing, re-read —
//! [`login`](crate::boundary::login) carries both rulings, and
//! [`Query::LoginTail`](crate::boundary::Query::LoginTail) is the lane.
