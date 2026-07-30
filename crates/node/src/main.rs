//! Thin binary entry point for the node (CHN-3).
//!
//! Loops forever: mint a block every `TICK_INTERVAL_SECS`, print the head,
//! sleep, repeat. All testable logic lives in `lib.rs`; this is just the
//! untested shell that drives real time.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use store::BlockStore;

fn main() {
    let mut store = BlockStore::new();
    let mut tick_no = 0u64;

    loop {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_millis() as u64;

        node::tick(&mut store, timestamp, tick_no);

        if let (Some(head), Some(head_digest)) = (store.head(), store.head_digest()) {
            println!("tick {tick_no}: head height={} digest={}", head.height, head_digest);
        }

        tick_no += 1;
        std::thread::sleep(Duration::from_secs(node::TICK_INTERVAL_SECS));
    }
}
