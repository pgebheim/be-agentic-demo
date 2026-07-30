//! Genesis block construction and parent linking (T3).
//!
//! `genesis()` produces the canonical height-0 block with no parent. `child_of`
//! extends a chain by one block, threading the parent's digest through
//! `parent_digest` so each block cryptographically commits to its predecessor.
//!
//! Canonical encoding contract (for `BlockExt::digest`): every field of
//! `Block` — `height`, `parent_digest`, `timestamp`, and each `Transaction`'s
//! `from`/`to`/`amount` — must be serialized in a fixed order, each variable-
//! length piece (strings, the transactions vector, the optional
//! `parent_digest`) prefixed with its length, and every integer written
//! little-endian. This must be injective (no two distinct `Block`s may
//! encode to the same bytes) so that `util::digest` over the encoding is a
//! faithful content hash. The digest is over this canonical encoding, NOT
//! over the derived `Hash` impl on `Block`.

use types::{Block, Digest, Transaction};

/// Unix timestamp (ms) assigned to the genesis block.
pub const GENESIS_TIMESTAMP: u64 = 0;

/// Build the canonical genesis block: height 0, no parent, no transactions.
pub fn genesis() -> Block {
    Block::new(0, None, GENESIS_TIMESTAMP, vec![])
}

/// Extension trait providing a content digest for `Block`.
///
/// This lives here (not as an inherent impl on `Block`) because the orphan
/// rule forbids implementing methods on a foreign type (`Block` is defined in
/// `types`) from a downstream crate; an extension trait sidesteps that.
pub trait BlockExt {
    /// SHA-256 digest (hex-encoded) of the block's canonical encoding.
    fn digest(&self) -> Digest;
}

impl BlockExt for Block {
    fn digest(&self) -> Digest {
        util::digest(&encode(self))
    }
}

/// Length-prefix `bytes` with its length as a little-endian `u64`.
fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    buf.extend_from_slice(bytes);
}

/// Canonical, injective, length-prefixed little-endian encoding of every
/// field of `block`. See the module-level doc comment for the contract.
fn encode(block: &Block) -> Vec<u8> {
    let mut buf = Vec::new();

    // Destructure exhaustively so that a field added to `Block` or
    // `Transaction` in the `types` crate forces a compile error here rather
    // than being silently omitted from the digest.
    let Block { height, parent_digest, timestamp, transactions } = block;

    buf.extend_from_slice(&height.to_le_bytes());

    match parent_digest {
        None => buf.push(0u8),
        Some(parent) => {
            buf.push(1u8);
            push_len_prefixed(&mut buf, parent.as_bytes());
        }
    }

    buf.extend_from_slice(&timestamp.to_le_bytes());

    buf.extend_from_slice(&(transactions.len() as u64).to_le_bytes());
    for transaction in transactions {
        let Transaction { from, to, amount } = transaction;
        push_len_prefixed(&mut buf, from.as_bytes());
        push_len_prefixed(&mut buf, to.as_bytes());
        buf.extend_from_slice(&amount.to_le_bytes());
    }

    buf
}

/// Build the next block in a chain, linking it to `parent` via
/// `parent.digest()`.
pub fn child_of(parent: &Block, timestamp: u64, transactions: Vec<Transaction>) -> Block {
    Block::new(
        parent.height.checked_add(1).expect("block height overflow"),
        Some(parent.digest()),
        timestamp,
        transactions,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(from: &str, to: &str, amount: u64) -> Transaction {
        Transaction { from: from.into(), to: to.into(), amount }
    }

    // --- Acceptance: parent linking ---------------------------------------

    #[test]
    fn child_links_to_genesis_parent_digest() {
        let genesis = genesis();
        let child = child_of(&genesis, 1, vec![tx("aa", "bb", 5)]);
        assert_eq!(child.parent_digest, Some(genesis.digest()));
    }

    #[test]
    fn three_block_chain_links_generalize() {
        let genesis = genesis();
        let child = child_of(&genesis, 1, vec![tx("aa", "bb", 5)]);
        let grandchild = child_of(&child, 2, vec![tx("bb", "cc", 7)]);
        assert_eq!(grandchild.parent_digest, Some(child.digest()));
    }

    // --- Genesis shape -------------------------------------------------

    #[test]
    fn genesis_has_height_zero() {
        assert_eq!(genesis().height, 0);
    }

    #[test]
    fn genesis_has_no_parent() {
        assert_eq!(genesis().parent_digest, None);
    }

    #[test]
    fn genesis_has_the_genesis_timestamp() {
        assert_eq!(genesis().timestamp, GENESIS_TIMESTAMP);
    }

    #[test]
    fn genesis_has_no_transactions() {
        assert!(genesis().transactions.is_empty());
    }

    // --- Child shape -----------------------------------------------------

    #[test]
    fn child_height_is_parent_height_plus_one() {
        let genesis = genesis();
        let child = child_of(&genesis, 1, vec![]);
        assert_eq!(child.height, genesis.height + 1);
    }

    // --- Determinism -------------------------------------------------------

    #[test]
    fn genesis_is_deterministic() {
        assert_eq!(genesis(), genesis());
    }

    #[test]
    fn genesis_digest_is_deterministic() {
        assert_eq!(genesis().digest(), genesis().digest());
    }

    #[test]
    fn genesis_digest_is_valid_hex() {
        assert!(util::is_hex(&genesis().digest()));
    }

    // --- Sensitivity: each field affects the digest ------------------------

    #[test]
    fn digest_is_sensitive_to_height() {
        let a = Block::new(0, None, 0, vec![]);
        let b = Block::new(1, None, 0, vec![]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_timestamp() {
        let a = Block::new(0, None, 0, vec![]);
        let b = Block::new(0, None, 1, vec![]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_parent_digest_none_vs_some() {
        let a = Block::new(1, None, 0, vec![]);
        let b = Block::new(1, Some("aa".into()), 0, vec![]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_parent_digest_value() {
        let a = Block::new(1, Some("aa".into()), 0, vec![]);
        let b = Block::new(1, Some("bb".into()), 0, vec![]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_added_transaction() {
        let a = Block::new(0, None, 0, vec![tx("aa", "bb", 5)]);
        let b = Block::new(0, None, 0, vec![tx("aa", "bb", 5), tx("cc", "dd", 9)]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_transaction_amount() {
        let a = Block::new(0, None, 0, vec![tx("aa", "bb", 5)]);
        let b = Block::new(0, None, 0, vec![tx("aa", "bb", 6)]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_transaction_order() {
        let a = Block::new(0, None, 0, vec![tx("aa", "bb", 5), tx("cc", "dd", 9)]);
        let b = Block::new(0, None, 0, vec![tx("cc", "dd", 9), tx("aa", "bb", 5)]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_transaction_to() {
        let a = Block::new(0, None, 0, vec![tx("aa", "bb", 0)]);
        let b = Block::new(0, None, 0, vec![tx("aa", "cc", 0)]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_transaction_from() {
        let a = Block::new(0, None, 0, vec![tx("aa", "cc", 0)]);
        let b = Block::new(0, None, 0, vec![tx("bb", "cc", 0)]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_field_framing() {
        // from/to are length-framed: "aa"+"bb" and "aab"+"b" share the same
        // concatenation and differ only because each field is length-prefixed.
        // Guards against a "simplification" that drops the length words.
        let a = Block::new(0, None, 0, vec![tx("aa", "bb", 0)]);
        let b = Block::new(0, None, 0, vec![tx("aab", "b", 0)]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_from_length_prefix() {
        // These collide iff `from`'s length prefix is dropped: "" + "\0" and
        // "\x01" + "" share the same 1-byte concatenation, so only the framing
        // length word distinguishes them. Pins the `from` prefix specifically.
        let a = Block::new(0, None, 0, vec![tx("", "\u{0}", 0)]);
        let b = Block::new(0, None, 0, vec![tx("\u{1}", "", 0)]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn digest_is_sensitive_to_to_length_prefix() {
        // Two distinct multi-tx blocks that collide iff `to`'s length prefix is
        // dropped: the per-tx `to` boundary is the only thing distinguishing
        // [("",""),("","\0")] from [("","\0"),("","")]. Pins the `to` prefix.
        let a = Block::new(0, None, 0, vec![tx("", "", 0), tx("", "\u{0}", 0)]);
        let b = Block::new(0, None, 0, vec![tx("", "\u{0}", 0), tx("", "", 0)]);
        assert_ne!(a.digest(), b.digest());
    }

    // --- Collision sanity ----------------------------------------------

    #[test]
    fn child_digest_differs_from_genesis_digest() {
        let genesis = genesis();
        let child = child_of(&genesis, 1, vec![tx("aa", "bb", 5)]);
        assert_ne!(child.digest(), genesis.digest());
    }
}
