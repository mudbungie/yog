//! **What a server binds** (REMOTE §15.3; bl-52ae), separated from what a
//! client dials. §8 made `address` one line that named both; they are two
//! facts, and this is the one place the first is derived from the second.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// **What an address asks the server to bind** (REMOTE §15.3). A loopback
/// address binds loopback on its port, exactly as stated — `127.0.0.1:0`, the
/// self-provisioned request for a kernel-chosen port, included. Anything else
/// binds **every interface** on the stated port: the host is what a client
/// dials and a certificate answers to, never what the socket is bound to, and
/// reach is the firewall's question.
///
/// **No name is resolved**, so a box whose resolver is down still serves: the
/// host is read as an address literal or as the one word `localhost`, and a
/// host that is neither is simply not loopback.
pub(crate) fn binding(address: &str) -> Result<SocketAddr, String> {
    let refusal = || format!("bind {address}: an address is host:port, the port a number");
    let (host, port) = address.rsplit_once(':').ok_or_else(refusal)?;
    let port: u16 = port.parse().map_err(|_| refusal())?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let ip = match host.parse::<IpAddr>() {
        Ok(ip) if ip.is_loopback() => ip,
        _ if host == "localhost" => Ipv4Addr::LOCALHOST.into(),
        _ => Ipv4Addr::UNSPECIFIED.into(),
    };
    Ok(SocketAddr::new(ip, port))
}

#[cfg(test)]
mod tests {
    use super::binding;

    /// A loopback address binds exactly what it states — the kernel-chosen
    /// `:0` included, in both families and under the one name read as loopback.
    #[test]
    fn a_loopback_address_binds_loopback_as_stated() {
        for (stated, bound) in [
            ("127.0.0.1:0", "127.0.0.1:0"),
            ("127.0.0.1:7737", "127.0.0.1:7737"),
            ("127.0.0.2:7737", "127.0.0.2:7737"),
            ("[::1]:7737", "[::1]:7737"),
            ("localhost:7737", "127.0.0.1:7737"),
        ] {
            assert_eq!(binding(stated).expect(stated).to_string(), bound);
        }
    }

    /// Anything else binds every interface on the stated port, and no name is
    /// resolved on the way: a name that resolves nowhere binds the same.
    #[test]
    fn any_other_host_binds_every_interface_on_the_stated_port() {
        for stated in [
            "engine.example:7737",
            "192.0.2.7:7737",
            "[2001:db8::7]:7737",
            "0.0.0.0:7737",
        ] {
            assert_eq!(binding(stated).expect(stated).to_string(), "0.0.0.0:7737");
        }
    }

    /// An address with no port, or a port that is no number, is a refusal that
    /// names the bind and the address.
    #[test]
    fn an_address_without_a_numeric_port_refuses() {
        for stated in ["engine.example", "engine.example:", "engine.example:wire"] {
            let refusal = binding(stated).expect_err(stated);
            assert!(refusal.contains(&format!("bind {stated}")), "{refusal}");
        }
    }
}
