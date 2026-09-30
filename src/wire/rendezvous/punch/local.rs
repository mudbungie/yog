//! Where this box would send from (REMOTE §13.2): the addresses presence
//! names beside the observed one. Split off [`punch`](super) at §12's budget
//! (bl-5276) — a fact about the box's routes, not about the punch.

use std::net::{IpAddr, UdpSocket};

/// The addresses this box would send from, one per family it has a route
/// on: a UDP socket "connected" to a global address sends nothing and reads
/// back the local end the route would use — so the address only has to
/// select the default route, and the documentation ranges (RFC 5737, RFC
/// 3849) do that as well as any real host would. Loopback never appears — a
/// box with no route has no address to publish, and says so with an empty
/// list.
pub(crate) fn local_ips() -> Vec<IpAddr> {
    ["[2001:db8::1]:53", "192.0.2.1:53"]
        .iter()
        .filter_map(|probe| {
            let bind = if probe.starts_with('[') {
                "[::]:0"
            } else {
                "0.0.0.0:0"
            };
            let socket = UdpSocket::bind(bind).ok()?;
            socket.connect(probe).ok()?;
            let ip = socket.local_addr().ok()?.ip();
            (!ip.is_loopback()).then_some(ip)
        })
        .collect()
}
