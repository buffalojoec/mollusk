//! Benches Mollusk invocation (instructions per second)
use {
    criterion::{criterion_group, criterion_main, Criterion, Throughput},
    mollusk::{result::Check, Mollusk},
    solana_sdk::{
        account::AccountSharedData, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
        system_instruction, system_program,
    },
    solana_system_program::system_processor::DEFAULT_COMPUTE_UNITS,
};

fn transfer_checked_unchecked(c: &mut Criterion) {
    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let base_lamports = 100 * LAMPORTS_PER_SOL;
    let transfer_amount = 1;

    let instruction = system_instruction::transfer(&sender, &recipient, transfer_amount);
    let accounts = vec![
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
        Check::compute_units(DEFAULT_COMPUTE_UNITS),
        Check::account(&sender)
            .lamports(base_lamports - transfer_amount)
            .build(),
        Check::account(&recipient)
            .lamports(base_lamports + transfer_amount)
            .build(),
    ];

    // No logs for bench
    let mollusk = Mollusk::default();
    solana_logger::setup_with("");

    // Create transfers group with elements/second
    let mut g = c.benchmark_group("transfers");
    g.throughput(Throughput::Elements(1));

    // Bench transfer with post-execution checks
    g.bench_function("transfer_checked", |b| {
        b.iter(|| {
            mollusk.process_and_validate_instruction(&instruction, accounts.clone(), &checks);
        })
    });

    // Bench transfer without post-execution checks
    g.bench_function("transfer_unchecked", |b| {
        b.iter(|| {
            mollusk.process_instruction(&instruction, accounts.clone());
        })
    });

    g.finish();
}

criterion_group!(transfers, transfer_checked_unchecked);
criterion_main!(transfers);
