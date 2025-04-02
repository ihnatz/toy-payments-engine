# Toy Payment Processor

A Rust-based payment engine that processes deposits, withdrawals, and disputes with thread-safe
concurrency. Implements fund verification to prevent invalid transactions (e.g., fraudulent
deposits) and tracks held/available balances. Supports graceful shutdown and outputs account
states in CSV format.

## Usage

```
$ cargo run -- transactions.csv > accounts.csv
```

## Input example

```csv
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
```

## Core Features

- Processes deposits, withdrawals, and disputes with thread-safe concurrency
- Handles negative balances when disputed deposits are withdrawn (e.g., fraudulent funds)
- Maintains a full audit trail in the ledger, including transactions on locked accounts


## Validation Notes:

- Negative Balances: Correctly models real-world banking behavior for disputed/reversed deposits.
- Ledger Integrity: Storing all operations (even on locked accounts) ensures replayability—critical for debugging and compliance.
- Edge Cases Covered: The design prevents "ghost withdrawals" by tracking held/available balances separately.


## Concurrency Model

The engine distributes work across isolated worker threads using per-client queues. Each worker:

- Processes only transactions for its assigned client IDs (via client_id % worker_count)
- Never accesses other workers' transactions or account states
- Achieves lock-free operation for uncontended cases via sharded account access

## Tradeoffs:

- No cross-worker contention (no locks between workers)
- Simple failure isolation (one client's errors won't block others)
- Uneven balance distribution possible (hot clients may bottleneck a single worker)
