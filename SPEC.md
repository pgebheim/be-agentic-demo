# SPEC — a minimal BFT blockchain

## Goal

Build a small but real Byzantine-fault-tolerant blockchain in Rust: a local
network of validators reaches consensus on a growing chain of blocks, and a
web explorer renders the chain as it finalizes. The point is a working
end-to-end pipeline — **types → chain rules → consensus → indexer → explorer** —
that we can stand up locally, not a production L1.

This repository starts with nothing but this spec. Everything below is built
from here, one ticket at a time.

## Non-goals

- No token economics, smart-contract VM, staking, or public testnet.
- No hand-rolled consensus — we compose an existing consensus library.
- No persistence guarantees beyond an in-memory store to start.

## Build order

The chain is built foundations-first. Milestone 0 is small, pure, and has no
external dependencies — the right place to start from a blank repo. Later
milestones are the north star.

### Milestone 0 — foundations (start here)

Small, pure, unit-testable Rust. These are the first tickets.

- **T1 · Core types.** A `types` crate defining the domain: `Digest` (a
  hex-encoded hash), `Transaction { from, to, amount }`, and
  `Block { height, parent_digest, timestamp, transactions }`. Include a
  constructor and derive the obvious traits. Unit-tested.
- **T2 · Hashing & hex util.** A `util` crate: hex encode/decode with
  validation, and a `digest(bytes) -> Digest` helper over a standard hash
  (e.g. SHA-256). Pure functions, unit-tested. No dependency on `types`.
- **T3 · Genesis & block linking.** Using T1 + T2: construct a deterministic
  genesis block, compute a block's own digest, and link a child to its parent
  by `parent_digest`. A test builds a two-block chain and verifies the link
  (child.parent_digest == genesis.digest). This is the first moment it is
  actually a *chain*.

### Milestone 1 — a chain that grows (single node)

A local chain that extends itself — no consensus yet. Three **interleaved**
tickets on one shared integration branch (this milestone is an *epic*):

- **CHN-1 · Block store.** An in-memory store: append a block, get by height and
  by digest, track the head. Unit-tested. (depends on Milestone 0 types)
- **CHN-2 · Chain-append.** Mint the next block from a set of transactions,
  linking `parent_digest` to the current head, and append it. A test appends
  several blocks and verifies the links form a chain. (depends on CHN-1 + util)
- **CHN-3 · Node.** A `node` binary that mints a block every few seconds and
  appends it — the chain grows on its own, printing the head each tick.
  (depends on CHN-2)

This is the milestone the demo **kicks off as an epic** and leaves running.

### Milestone 2 — consensus core (north star)

Compose a BFT consensus library so ≥4 local validators agree on block order;
finalized blocks are reported in a deduped, ordered stream.

### Milestone 3 — validator node

A `validator` binary: dev keypair/identity, engine boot, and a script to run a
local cluster of validators that finalizes a growing chain.

### Milestone 4 — indexer

Consume finalized blocks and serve them over a REST API (latest / by-height /
by-digest) plus a WebSocket stream of new finalizations.

### Milestone 5 — explorer

A small web UI that renders the live chain from the indexer: a list of recent
blocks, a block detail view, and a live-head indicator.

## Success criteria

1. `cargo build` succeeds; `cargo test` is green.
2. A local network finalizes a growing chain of real blocks.
3. The explorer shows blocks appearing in real time.

> Iterating from Milestone 0 to a running chain is the exercise. The workflow
> — a shared board, specs over steering, and skills with gates — is what
> carries you between them.
