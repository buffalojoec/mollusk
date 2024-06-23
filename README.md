# Mollusk

SVM program test harness.

## Harness

The harness is designed to directly invoke the loaded executable program using
the BPF Loader, bypassing any transaction sanitization and runtime checks, and
instead directly processing the instruction with the BPF Loader.

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
    (key1, AccountSharedData::new(10_000, 0, &system_program::id())),
    (key2, AccountSharedData::new(10_000, 0, &system_program::id())),
];

let mollusk = Mollusk::new(program_id, "my_program");

let result = mollusk.process_instruction(&instruction, &accounts);
```

You can also use the `Checks` API provided by Mollusk for easy post-execution
checks, rather than writing them manually. The API method
`process_and_validate_instruction` will still return the result, allowing you
to perform further checks if you desire.

> Note: `Mollusk::default()` will use the System program as the program to
> invoke.

```rust
let sender = Pubkey::new_unique();
let recipient = Pubkey::new_unique();

let base_lamports = 100_000_000u64;
let transfer_amount = 42_000u64;

let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);
let accounts = [
    (
        sender,
        AccountSharedData::new(base_lamports, 0, &system_program::id()),
    ),
    (
        recipient,
        AccountSharedData::new(base_lamports, 0, &system_program::id()),
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

Mollusk::default().process_and_validate_instruction(&instruction, &accounts, &checks);
```

## Bencher

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
    .bench(("bench4", &instruction4, &accounts4))
    .bench(("bench5", &instruction5, &accounts5))
    .bench(("bench6", &instruction6, &accounts6))
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
| bench1 | 579 | +129 |
| bench2 | 1,204 | +754 |
| bench3 | 2,811 | +2,361 |
