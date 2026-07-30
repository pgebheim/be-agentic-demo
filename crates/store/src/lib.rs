//! In-memory block store (CHN-1).
//!
//! `BlockStore` computes each block's digest itself from a canonical byte
//! encoding of the block's fields (via `util::digest`), so "get by digest"
//! keys on that computed digest, not on anything the caller supplies.
//! Blocks are retrievable by height and by digest, and the store tracks the
//! `head` (the block with the greatest height).

use std::collections::HashMap;

use types::{Block, Digest};

/// An in-memory, append-only store of blocks.
#[derive(Debug, Default)]
pub struct BlockStore {
    by_digest: HashMap<Digest, Block>,
    by_height: HashMap<u64, Digest>,
    head_height: Option<u64>,
}

impl BlockStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a block to the store, returning its computed digest.
    ///
    /// The digest is computed from a canonical encoding of the block's
    /// fields (see [`canonical_encoding`]), not supplied by the caller.
    pub fn append(&mut self, block: Block) -> Digest {
        let digest = util::digest(&canonical_encoding(&block));
        let height = block.height;
        self.by_digest.insert(digest.clone(), block);
        self.by_height.insert(height, digest.clone());
        self.head_height = Some(self.head_height.map_or(height, |h| h.max(height)));
        digest
    }

    /// Look up a block by its height (the most recently appended block at
    /// that height, if more than one has been appended there).
    pub fn get_by_height(&self, height: u64) -> Option<&Block> {
        let digest = self.by_height.get(&height)?;
        self.by_digest.get(digest)
    }

    /// Look up a block by its computed digest (hex string).
    pub fn get_by_digest(&self, digest: &str) -> Option<&Block> {
        self.by_digest.get(digest)
    }

    /// The block with the greatest height (the chain tip), if any.
    pub fn head(&self) -> Option<&Block> {
        let height = self.head_height?;
        self.get_by_height(height)
    }

    /// Number of distinct blocks in the store.
    pub fn len(&self) -> usize {
        self.by_digest.len()
    }

    /// True if the store has no blocks.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Canonical byte encoding of a block's fields, used as the input to
/// `util::digest` when computing a block's digest.
///
/// Every field is length-prefixed (as a big-endian `u64` byte count) before
/// its bytes so that no ambiguity can arise between adjacent fields (e.g. a
/// `from` of "ab" + "cd" vs. "a" + "bcd").
fn canonical_encoding(block: &Block) -> Vec<u8> {
    let mut out = Vec::new();

    push_bytes(&mut out, &block.height.to_be_bytes());

    match &block.parent_digest {
        Some(parent) => {
            push_bytes(&mut out, &[1]);
            push_bytes(&mut out, parent.as_bytes());
        }
        None => push_bytes(&mut out, &[0]),
    }

    push_bytes(&mut out, &block.timestamp.to_be_bytes());

    push_bytes(&mut out, &(block.transactions.len() as u64).to_be_bytes());
    for tx in &block.transactions {
        push_bytes(&mut out, tx.from.as_bytes());
        push_bytes(&mut out, tx.to.as_bytes());
        push_bytes(&mut out, &tx.amount.to_be_bytes());
    }

    out
}

/// Append `bytes` to `out`, prefixed with its length as a big-endian `u64`.
fn push_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    out.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Transaction;

    fn tx(from: &str, to: &str, amount: u64) -> Transaction {
        Transaction { from: from.into(), to: to.into(), amount }
    }

    fn genesis() -> Block {
        Block::new(0, None, 1_700_000_000, vec![tx("aa", "bb", 1)])
    }

    fn block_at(height: u64, parent: Option<Digest>) -> Block {
        Block::new(height, parent, 1_700_000_000 + height, vec![tx("cc", "dd", height)])
    }

    #[test]
    fn new_store_is_empty_with_no_head() {
        let store = BlockStore::new();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
        assert!(store.head().is_none());
    }

    #[test]
    fn append_then_get_by_height_returns_same_block() {
        let mut store = BlockStore::new();
        let block = genesis();
        store.append(block.clone());
        assert_eq!(store.get_by_height(0), Some(&block));
    }

    #[test]
    fn append_then_get_by_digest_returns_same_block() {
        let mut store = BlockStore::new();
        let block = genesis();
        let digest = store.append(block.clone());
        assert_eq!(store.get_by_digest(&digest), Some(&block));
    }

    #[test]
    fn returned_digest_is_valid_sha256_hex() {
        let mut store = BlockStore::new();
        let digest = store.append(genesis());
        assert_eq!(digest.len(), 64, "sha256 hex digest should be 64 chars");
        assert!(
            digest.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "digest should be lowercase hex: {digest}"
        );
    }

    #[test]
    fn get_by_height_returns_none_for_absent_height() {
        let mut store = BlockStore::new();
        store.append(genesis());
        assert!(store.get_by_height(42).is_none());
    }

    #[test]
    fn get_by_digest_returns_none_for_absent_digest() {
        let mut store = BlockStore::new();
        store.append(genesis());
        assert!(store.get_by_digest("not-a-real-digest").is_none());
    }

    #[test]
    fn head_is_none_on_empty_store() {
        let store = BlockStore::new();
        assert!(store.head().is_none());
    }

    #[test]
    fn head_returns_highest_height_block_after_appends() {
        let mut store = BlockStore::new();
        let b0 = genesis();
        let d0 = store.append(b0.clone());
        let b1 = block_at(1, Some(d0));
        store.append(b1.clone());
        assert_eq!(store.head(), Some(&b1));
    }

    #[test]
    fn two_different_blocks_produce_two_different_digests_and_both_are_retrievable() {
        let mut store = BlockStore::new();
        let b0 = genesis();
        let d0 = store.append(b0.clone());
        let b1 = block_at(1, Some(d0.clone()));
        let d1 = store.append(b1.clone());

        assert_ne!(d0, d1, "distinct blocks must have distinct digests");
        assert_eq!(store.get_by_digest(&d0), Some(&b0));
        assert_eq!(store.get_by_digest(&d1), Some(&b1));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn duplicate_height_append_keeps_digests_consistent() {
        let mut store = BlockStore::new();
        let a = block_at(5, None);
        let d_a = store.append(a.clone());
        let b = Block::new(5, None, 1_700_000_000 + 5, vec![tx("ee", "ff", 99)]);
        let d_b = store.append(b.clone());

        assert_ne!(d_a, d_b, "distinct blocks at the same height must have distinct digests");
        assert_eq!(
            store.get_by_digest(&d_a),
            Some(&a),
            "get_by_digest(d_a) must return the block whose digest is d_a, not the block that later overwrote its height"
        );
        assert_eq!(store.get_by_digest(&d_b), Some(&b));
        assert_eq!(
            store.get_by_height(5),
            Some(&b),
            "get_by_height should return the most-recently-appended block at that height"
        );
    }

    #[test]
    fn appending_blocks_out_of_height_order_still_tracks_correct_head() {
        let mut store = BlockStore::new();
        let b0 = genesis();
        let d0 = store.append(b0.clone());
        let b2 = block_at(2, Some(d0.clone()));
        store.append(b2.clone());
        // Append a lower-height block after a higher one.
        let b1 = block_at(1, Some(d0));
        store.append(b1);

        assert_eq!(store.head(), Some(&b2));
    }
}
