//! Core domain types for the chain (T1).
//!
//! Mirrors the shape a block explorer would render: a `Block` carries a height,
//! its parent's digest, a timestamp, and a payload of transactions. `Digest` is
//! a hex-encoded hash (see the `util` crate for the hashing itself).

/// A digest is a hex-encoded hash, no `0x` prefix.
pub type Digest = String;

/// A single transaction: an opaque, addressed payload (no VM, no token model).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    /// Hex-encoded sender address.
    pub from: String,
    /// Hex-encoded recipient address.
    pub to: String,
    /// Opaque payload value.
    pub amount: u64,
}

/// A block in the chain.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    /// Number of blocks since genesis.
    pub height: u64,
    /// Parent block's digest; `None` for genesis.
    pub parent_digest: Option<Digest>,
    /// Unix timestamp (ms) the block was produced.
    pub timestamp: u64,
    /// The block's payload.
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Construct a block from its parts.
    pub fn new(
        height: u64,
        parent_digest: Option<Digest>,
        timestamp: u64,
        transactions: Vec<Transaction>,
    ) -> Self {
        Self { height, parent_digest, timestamp, transactions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_block_with_one_tx() {
        let tx = Transaction { from: "aa".into(), to: "bb".into(), amount: 5 };
        let block = Block::new(1, Some("genesis".into()), 1_700_000_000, vec![tx.clone()]);
        assert_eq!(block.height, 1);
        assert_eq!(block.parent_digest.as_deref(), Some("genesis"));
        assert_eq!(block.transactions.len(), 1);
        assert_eq!(block.transactions[0].amount, 5);
    }
}
