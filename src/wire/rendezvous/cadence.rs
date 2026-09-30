//! **Every duration the loop keeps, and where it starts walking** (REMOTE
//! §13.7 ruling 3): the stated defaults, each a field a test shortens. Split
//! off the root at §12's budget (bl-5276) when the punch port gained its
//! standing acceptor.

use crate::wire::server::Quiet;
use std::time::Duration;

/// The mainline DHT's bootstrap nodes — the caller's fact, resolved on the
/// loop's own thread (a name lookup is a network act and boot is not). The
/// four standard long-lived routers, because from the deployed engine box
/// only one of the first two answered at all (REMOTE §13.7 ruling 3,
/// bl-9408): a silent router should cost a quarter of the roster, not half.
pub(crate) fn mainline() -> Vec<String> {
    vec![
        "router.bittorrent.com:6881".to_owned(),
        "dht.transmissionbt.com:6881".to_owned(),
        "router.utorrent.com:6881".to_owned(),
        "dht.aelitis.com:6881".to_owned(),
    ]
}

/// Every duration the loop keeps — stated defaults (REMOTE §13.7 ruling 3), and a
/// test's to shorten.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cadence {
    /// How often presence is republished against the DHT's storage decay.
    pub(crate) publish: Duration,
    /// How often the inbox is read.
    pub(crate) poll: Duration,
    /// How long the loop sleeps between looks at the clock.
    pub(crate) tick: Duration,
    /// How long a punch keeps sending SYNs.
    pub(crate) window: Duration,
    /// How a punched connection's silence is read.
    pub(crate) quiet: Quiet,
}

impl Default for Cadence {
    fn default() -> Cadence {
        Cadence {
            publish: Duration::from_hours(1),
            poll: Duration::from_secs(15),
            tick: Duration::from_secs(1),
            window: Duration::from_secs(20),
            quiet: Quiet::held(),
        }
    }
}
