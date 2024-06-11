# Mollusk

Solana program testing tools.

## Harness

The harness is designed to directly invoke the loaded executable program using
the BPF Loader, bypassing any transaction sanitization and runtime checks, and
instead directly processing the instruction with the BPF Loader.

Example:

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

let result = mollusk.process_instruction(instruction, accounts);
```

## Bencher

Mollusk also offers a compute unit usage bencher for profiling a program's
compute unit usage.

Example:

```rust
MolluskComputeUnitBencher::new(mollusk)
    .benchmark(BENCHMARK_COMPUTE_UNITS)
    .bench("bench1", instruction1, accounts1)
    .bench("bench2", instruction2, accounts2)
    .bench("bench3", instruction3, accounts3)
    .iterations(100)
    .must_pass(true)
    .out_dir("../target/benches")
    .execute();
```

You can invoke this benchmark test with `cargo bench`.

Mollusk will output bench details to the output directory in both JSON and
Markdown.

Markdown example:

| Name | Median | Mark Delta |
|------|--------|------------|
| bench1 | 579 | +129 |
| bench2 | 1,204 | +754 |
| bench3 | 2,811 | +2,361 |
