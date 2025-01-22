# Mollusk

SVM program test harness.

## Harness

The harness is designed to directly invoke the loaded executable program using
the rBPF VM, bypassing any transaction sanitization and runtime checks, and
instead directly processing the instruction with the VM.

```rust
let program_id = Pubkey::new_unique();
let key1 = Pubkey::new_unique();
let key2 = Pubkey::new_unique();

let instruction = Instruction::new_with_bytes(
    program_id,
    &[],
    vec![
        AccountMeta::new(key1, false),
        AccountMeta::new_readonly(key2, false),
    ],
);

let accounts = vec![
    (key1, Account::default()),
    (key2, Account::default()),
];

let mollusk = Mollusk::new(program_id, "my_program");

let result = mollusk.process_instruction(&instruction, &accounts);
```

You can also use the `Check` API provided by Mollusk for easy post-execution
checks, rather than writing them manually. The API method
`process_and_validate_instruction` will still return the result, allowing you
to perform further checks if you desire.

```rust
let sender = Pubkey::new_unique();
let recipient = Pubkey::new_unique();

let base_lamports = 100_000_000u64;
let transfer_amount = 42_000u64;

let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);
let accounts = [
    (
        sender,
        Account::new(base_lamports, 0, &system_program::id()),
    ),
    (
        recipient,
        Account::new(base_lamports, 0, &system_program::id()),
    ),
];
let checks = vec![
    Check::success(),
    Check::compute_units(system_processor::DEFAULT_COMPUTE_UNITS),
    Check::account(&sender)
        .lamports(base_lamports - transfer_amount)
        .build(),
    Check::account(&recipient)
        .lamports(base_lamports + transfer_amount)
        .build(),
];

Mollusk::default().process_and_validate_instruction(
    &instruction,
    &accounts,
    &checks,
);
```

## Compute Unit Bencher

Mollusk also offers a compute unit usage bencher for profiling a program's
compute unit usage.

Example:

```rust
// If using with `cargo bench`, tell Mollusk where to find the program.
std::env::set_var("SBF_OUT_DIR", "../target/deploy");

// Optionally disable logging.
solana_logger::setup_with("");

/* Instruction & accounts setup ... */

let mollusk = Mollusk::new(&program_id, "my_program");

MolluskComputeUnitBencher::new(mollusk)
    .bench(("bench0", &instruction0, &accounts0))
    .bench(("bench1", &instruction1, &accounts1))
    .bench(("bench2", &instruction2, &accounts2))
    .bench(("bench3", &instruction3, &accounts3))
    .must_pass(true)
    .out_dir("../target/benches")
    .execute();
```

You can invoke this benchmark test with `cargo bench`. Don't forget to add a
bench to your project's `Cargo.toml`.

```toml
[[bench]]
name = "compute_units"
harness = false
```

Mollusk will output bench details to the output directory in Markdown.

> Note: `Delta` is the change since the last time the bench was run.

| Name | CUs | Delta |
|------|--------|------------|
| bench0 | 450 | -- |
| bench1 | 579 | -129 |
| bench2 | 1,204 | +754 |
| bench3 | 2,811 | +2,361 |

## Fuzz Fixture Support

Mollusk also has first-class support for generating fixtures from tests, which
can be used for things like fuzzing.

There are two protobuf layouts supported by Mollusk:
* [`org.mollusk.svm`](./fuzz/fixture/proto): The protobuf layouts defined by
  the Mollusk library, which map directly to the structure of a Mollusk unit
  test.
* `org.solana.sealevel.v1`: The protobuf layouts defined by Firedancer and used
  to test program instructions between targets on Firedancer and Agave.

The first (`mollusk-svm-fuzz-fixture`) is a crate defined alongside the Mollusk
library, and the other (`solana-svm-fuzz-harness-fixture`) comes from Agave's
SVM stack.

The base library itself (`mollusk-svm`) provides support for working with
fixtures directly from a Mollusk instance, via the `fuzz` and `fuzz-fd` feature
flags, which can be used standalone or together.

When either fuzz-fixture feature flag is enabled, Mollusk can do the following:
* Generate a fixture from a unit test.
* Process a given fixture as a unit test.
* Convert to and from fixtures to Mollusk tests and results.

To generate a fuzz fixture from a Mollusk unit test, provide the necessary
environment variables alongside your call to `cargo test-sbf`, like so:

```
EJECT_FUZZ_FIXTURES="my/fixtures/dir" cargo test-sbf ...
```

JSON versions of fixtures are also supported.

See the documentation in [`harness/src/lib.rs`](./harness/src/lib.rs) for more
information.
