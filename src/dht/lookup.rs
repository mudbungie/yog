//! The iterative lookup (BEP 5), the one walk every read and write of the
//! commons makes: ask the closest nodes you know, learn closer ones from
//! their answers, repeat until the K closest have all been asked. Rounds are
//! synchronous — α queries out, then one bounded wait for whatever answers
//! (`round`) — and a node that stays silent is never asked again and leaves
//! the frontier. Bounded twice over against a hostile commons: a round ends
//! at its deadline whatever is still pending, and the walk ends at
//! `max_queries` however many "closer" nodes the answers keep inventing. Each node that answers may also say
//! where it saw the query come from; the walk keeps those claims, one per
//! answering node, for [`Dht::observed`] to vote on.

use super::Dht;
use super::bencode::{Dict, bytes, entry};
use super::krpc::{self, Message, Node, NodeId};
use super::round::Pending;
use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;

/// What a walk found: every reply, closest first, and every error a node
/// answered instead of a reply.
#[derive(Debug, Default)]
pub(crate) struct Outcome {
    pub(crate) replies: Vec<(Node, Dict)>,
    pub(crate) errors: Vec<String>,
}

impl Dht {
    /// Walk toward `target` asking `q` of every node on the way — except the
    /// bootstrap, which is asked `find_node` whatever `q` is. The mainline's
    /// routers answer `find_node` and never BEP 44's `get` (REMOTE §13.7
    /// ruling 3, bl-f6e1), so a walk that asked them `q` was dark in one
    /// round; the bootstrap is a door into the keyspace, and every node it
    /// opens onto is asked `q`. One walk, not a lookup and then a second one.
    ///
    /// The bootstrap is a door and never a result (bl-9408): its answers seed
    /// the pool and its `ip` claims vote, but it is not a node near the target
    /// and nothing it says is in the [`Outcome`]. So a walk whose learned
    /// nodes were all silent — measured one walk in five from the deployed
    /// engine box, the one answering router naming a single node eight times —
    /// is the same `Err` as a silent bootstrap: a dark commons. Not an empty
    /// `Ok`, because an empty `Ok` already means *nodes near the target
    /// answered and held nothing*, and a caller seeding on it would be dark
    /// without being told. `Err` is that, or the socket itself failing; a
    /// walk whose learned nodes drew only errors is an `Ok` with none.
    ///
    /// A node is one `(id, address)`: a repeated entry is learned once, and
    /// one id at two addresses is two nodes to ask.
    ///
    /// The walk converges on a frontier, not on the pool (bl-d00f): the K
    /// closest nodes that replied or are not yet asked. A node asked and
    /// silent, or that answered only an error (live, a `get` refused as an
    /// unknown query), leaves it, so the walk ends when the K closest
    /// *responsive* nodes have all been asked — or at `max_queries` — never
    /// because dead nodes sat on the slots a live one past them needed. A
    /// node this socket cannot send to (a v6 address from a v4 socket) leaves
    /// it the same way, before the query is spent: a refused send is not
    /// counted and the round's slot goes to the next node.
    ///
    /// The door is asked again whenever the frontier runs dry before K nodes
    /// past it have replied — none, the dark case, being the one measured —
    /// as long as it has ever named anyone; `max_queries` bounds it. Measured from the deployed engine box (REMOTE §13.7 ruling
    /// 3), the one router that answers names a single random node eight times
    /// per query — often the same one several queries running — so the two
    /// seeds a round yields are both silent about one walk in three, and
    /// re-asking draws fresh ones for a round's wait. Asking only while the
    /// door named someone NEW was tried and measured: its repeats ended a
    /// walk dark that the next ask would have opened. A door that is silent
    /// or names nobody leaves the pool empty and is not re-asked, so a dark
    /// commons stays one round.
    pub(crate) fn search(&mut self, target: NodeId, q: &str) -> Result<Outcome, String> {
        if self.bootstrap.is_empty() {
            return Err("no bootstrap node to ask".into());
        }
        let mut asked: BTreeSet<SocketAddr> = BTreeSet::new();
        let mut replied: BTreeSet<SocketAddr> = BTreeSet::new();
        let mut pool: BTreeMap<([u8; 20], SocketAddr), Node> = BTreeMap::new();
        let mut out = Outcome::default();
        let mut claims = Vec::new();
        let mut sent = 0usize;
        let args = Dict::from([entry("target", bytes(&target.0))]);
        let mut picks = self.bootstrap.clone();
        let mut seeding = true;
        loop {
            let verb = if seeding { "find_node" } else { q };
            let quota = if seeding {
                picks.len()
            } else {
                self.config.alpha
            };
            let mut pending = Pending::new();
            for addr in picks {
                if pending.len() >= quota {
                    break;
                }
                asked.insert(addr);
                self.ask(&mut pending, addr, verb, args.clone());
            }
            sent += pending.len();
            self.collect(&mut pending, &mut |addr, message| match message {
                Message::Reply { r, ip, .. } => {
                    let Some(id) = r
                        .get(b"id".as_slice())
                        .and_then(|v| v.as_bytes())
                        .and_then(NodeId::parse)
                    else {
                        return;
                    };
                    let node = Node { id, addr };
                    claims.extend(ip);
                    let this = (!seeding).then_some(node);
                    for near in krpc::nodes_of(&r).into_iter().chain(this) {
                        pool.insert((near.id.distance(&target), near.addr), near);
                    }
                    if !seeding {
                        replied.insert(addr);
                        out.replies.push((node, r));
                    }
                }
                Message::Error { code, message, .. } if !seeding => {
                    out.errors.push(format!("{addr}: {code} {message}"));
                }
                Message::Error { .. } => {}
            })?;
            // The frontier: the K closest nodes that answered or are still
            // to ask. A node asked and silent — or one this socket could not
            // send to — has left it, so it never holds a slot a live node
            // past it would take.
            picks = pool
                .values()
                .filter(|n| replied.contains(&n.addr) || !asked.contains(&n.addr))
                .take(self.config.k)
                .filter(|n| !asked.contains(&n.addr))
                .map(|n| n.addr)
                .collect();
            seeding = picks.is_empty() && !pool.is_empty() && replied.len() < self.config.k;
            if seeding {
                picks.clone_from(&self.bootstrap);
            }
            if picks.is_empty() || sent >= self.config.max_queries {
                break;
            }
        }
        self.claims = claims;
        if out.replies.is_empty() && out.errors.is_empty() {
            return Err(format!("no DHT node answered {q} for {target}"));
        }
        out.replies.sort_by_key(|(n, _)| n.id.distance(&target));
        Ok(out)
    }
}
