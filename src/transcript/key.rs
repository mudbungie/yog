//! **A transcript row's stable identity** — `tx/<entry filename>#<block
//! index>`, the one spelling of the address a seat and this server both name a
//! row by.
//!
//! It was the row projection's own helper until bl-7942 severed the window.
//! The projection went with the window (a seat paints rows; a server answers
//! entries), but the key did not: the §7.3 step spine's placement derivation
//! ([`crate::rail::place`]) tells a seat which row each rule is drawn above,
//! and it says so in this vocabulary. One spelling of the identity, not two —
//! which is why the format lives here rather than in either end.

/// Key namespace for a transcript row.
const KEY_ROOT: &str = "tx";

/// A row's stable identity: the entry's filename and the block ordinal.
pub(crate) fn key(name: &str, block: usize) -> String {
    format!("{KEY_ROOT}/{name}#{block}")
}

#[cfg(test)]
mod tests {
    use super::key;

    /// The namespace, the filename and the ordinal, in that order — the shape
    /// the rail's placements and a seat's rows both read.
    #[test]
    fn a_key_names_its_entry_and_its_block() {
        assert_eq!(key("003-tool.json", 0), "tx/003-tool.json#0");
        assert_eq!(key("003-tool.json", 2), "tx/003-tool.json#2");
    }

    /// Two blocks of one entry are two identities, and two entries never
    /// collide at the same ordinal.
    #[test]
    fn keys_separate_blocks_and_entries() {
        assert_ne!(key("003-tool.json", 0), key("003-tool.json", 1));
        assert_ne!(key("003-tool.json", 0), key("004-tool.json", 0));
    }
}
