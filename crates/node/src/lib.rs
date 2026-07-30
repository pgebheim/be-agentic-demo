//! Node logic (CHN-3): mint a block every few seconds.
//!
//! The binary (`main.rs`) is a thin loop that sleeps `TICK_INTERVAL_SECS`
//! between calls to [`tick`], printing the head after each one. All of the
//! testable logic lives here so it can be exercised without real time
//! passing (tests pass explicit timestamps instead of calling
//! `SystemTime::now`).

use store::BlockStore;
use types::{Digest, Transaction};

/// How often the node mints a new block, in seconds.
pub const TICK_INTERVAL_SECS: u64 = 2;

/// A small, deterministic set of transactions for a given tick number, so
/// that successive ticks produce different blocks.
pub fn sample_transactions(tick: u64) -> Vec<Transaction> {
    vec![Transaction {
        from: format!("{tick:064x}"),
        to: format!("{:064x}", tick + 1),
        amount: tick,
    }]
}

/// Mint one block into `store` at `timestamp`, using
/// `sample_transactions(tick_no)` as its payload, and return the minted
/// block's digest.
pub fn tick(store: &mut BlockStore, timestamp: u64, tick_no: u64) -> Digest {
    store.mint(timestamp, sample_transactions(tick_no))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_tick_into_empty_store_produces_genesis_block() {
        let mut store = BlockStore::new();
        let digest = tick(&mut store, 1000, 0);

        let genesis = store.get_by_height(0).expect("genesis should be at height 0");
        assert_eq!(genesis.height, 0);
        assert_eq!(genesis.parent_digest, None);
        assert_eq!(store.get_by_digest(&digest), Some(genesis));
    }

    #[test]
    fn repeated_ticks_grow_a_contiguous_linked_chain() {
        let mut store = BlockStore::new();
        let mut digests = Vec::new();
        for n in 0..5u64 {
            let d = tick(&mut store, 1000 * (n + 1), n);
            digests.push(d);
        }

        assert_eq!(store.len(), 5);

        for n in 0..5u64 {
            let block = store.get_by_height(n).unwrap_or_else(|| panic!("missing height {n}"));
            assert_eq!(block.height, n);
            if n == 0 {
                assert_eq!(block.parent_digest, None, "genesis must have no parent");
            } else {
                assert_eq!(
                    block.parent_digest,
                    Some(digests[(n - 1) as usize].clone()),
                    "block at height {n} must link to the digest of the block one height below it"
                );
            }
        }
    }

    #[test]
    fn head_after_n_ticks_is_the_last_minted_block() {
        let mut store = BlockStore::new();
        let mut last_digest = None;
        for n in 0..5u64 {
            last_digest = Some(tick(&mut store, 1000 * (n + 1), n));
        }
        let last_digest = last_digest.expect("at least one tick");

        let head = store.head().expect("store should have a head after ticks");
        assert_eq!(head.height, 4);
        assert_eq!(store.head_digest(), Some(&last_digest));
    }

    #[test]
    fn sample_transactions_is_non_empty() {
        assert!(!sample_transactions(0).is_empty());
    }

    #[test]
    fn sample_transactions_differs_across_ticks() {
        assert_ne!(sample_transactions(0), sample_transactions(1));
        assert_ne!(sample_transactions(1), sample_transactions(2));
        assert_ne!(sample_transactions(0), sample_transactions(41));
    }
}
