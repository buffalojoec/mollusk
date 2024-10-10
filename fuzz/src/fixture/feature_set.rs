//! A Solana runtime feature set, as represented in the Solana SDK.

use {
    super::proto,
    solana_sdk::{feature_set::*, pubkey::Pubkey},
};

fn discriminator(pubkey: &Pubkey) -> u64 {
    u64::from_le_bytes(pubkey.to_bytes()[..8].try_into().unwrap())
}

fn create_feature_set<'a>(
    declared: impl Iterator<Item = &'a Pubkey>,
    evaluate: impl Fn(&Pubkey) -> bool,
) -> FeatureSet {
    let mut feature_set = FeatureSet::default();
    for id in declared {
        if evaluate(id) {
            feature_set.activate(id, 0);
        }
    }
    feature_set
}

#[derive(Debug)]
pub struct FixtureFeatureSet {
    pub features: Vec<u64>,
}

impl Default for FixtureFeatureSet {
    fn default() -> Self {
        Self::from(&FeatureSet::all_enabled())
    }
}

impl From<proto::FeatureSet> for FixtureFeatureSet {
    fn from(input: proto::FeatureSet) -> Self {
        Self {
            features: input.features,
        }
    }
}

impl From<FixtureFeatureSet> for FeatureSet {
    fn from(input: FixtureFeatureSet) -> Self {
        create_feature_set(FeatureSet::default().inactive.iter(), |id| {
            input.features.contains(&discriminator(id))
        })
    }
}

impl From<&FeatureSet> for FixtureFeatureSet {
    fn from(input: &FeatureSet) -> Self {
        let features = input.active.keys().map(discriminator).collect();
        Self { features }
    }
}

// Lists of agave supported feature flags, as of `2.0.13`.
// Inactive on all clusters.
static AGAVE_FEATURES_INACTIVE: &[Pubkey] = &[
    full_inflation::devnet_and_testnet::id(),
    zk_token_sdk_enabled::id(),
    stake_redelegate_instruction::id(),
    enable_partitioned_epoch_reward::id(),
    partitioned_epoch_rewards_superfeature::id(),
    stake_minimum_delegation_for_rewards::id(),
    skip_rent_rewrites::id(),
    loosen_cpi_size_restriction::id(),
    disable_turbine_fanout_experiments::id(),
    enable_big_mod_exp_syscall::id(),
    apply_cost_tracker_during_replay::id(),
    bpf_account_data_direct_mapping::id(),
    include_loaded_accounts_data_size_in_fee_calculation::id(),
    remaining_compute_units_syscall_enabled::id(),
    enable_program_runtime_v2_and_loader_v4::id(),
    disable_rent_fees_collection::id(),
    enable_zk_transfer_with_fee::id(),
    add_new_reserved_account_keys::id(),
    enable_zk_proof_from_account::id(),
    cost_model_requested_write_lock_cost::id(),
    chained_merkle_conflict_duplicate_proofs::id(),
    remove_rounding_in_fee_calculation::id(),
    enable_tower_sync_ix::id(),
    reward_full_priority_fee::id(),
    get_sysvar_syscall_enabled::id(),
    abort_on_invalid_curve::id(),
    migrate_feature_gate_program_to_core_bpf::id(),
    vote_only_full_fec_sets::id(),
    migrate_config_program_to_core_bpf::id(),
    enable_get_epoch_stake_syscall::id(),
    migrate_address_lookup_table_program_to_core_bpf::id(),
    zk_elgamal_proof_program_enabled::id(),
    move_stake_and_move_lamports_ixs::id(),
    ed25519_precompile_verify_strict::id(),
    verify_retransmitter_signature::id(),
    vote_only_retransmitter_signed_fec_sets::id(),
];
// Active on testnet.
static AGAVE_FEATURES_TESTNET: &[Pubkey] = &[];
// Active on devnet.
static AGAVE_FEATURES_DEVNET: &[Pubkey] = &[
    blake3_syscall_enabled::id(),
    libsecp256k1_fail_on_bad_count::id(),
    increase_tx_account_lock_limit::id(),
    timely_vote_credits::id(),
    allow_commission_decrease_at_any_time::id(),
    enable_chained_merkle_shreds::id(),
];
// Active on mainnet-beta.
static AGAVE_FEATURES_MAINNET_BETA: &[Pubkey] = &[
    deprecate_rewards_sysvar::id(),
    pico_inflation::id(),
    full_inflation::mainnet::certusone::vote::id(),
    full_inflation::mainnet::certusone::enable::id(),
    secp256k1_program_enabled::id(),
    spl_token_v2_multisig_fix::id(),
    no_overflow_rent_distribution::id(),
    filter_stake_delegation_accounts::id(),
    require_custodian_for_locked_stake_authorize::id(),
    spl_token_v2_self_transfer_fix::id(),
    warp_timestamp_again::id(),
    check_init_vote_data::id(),
    secp256k1_recover_syscall_enabled::id(),
    system_transfer_zero_check::id(),
    dedupe_config_program_signers::id(),
    verify_tx_signatures_len::id(),
    vote_stake_checked_instructions::id(),
    rent_for_sysvars::id(),
    libsecp256k1_0_5_upgrade_enabled::id(),
    tx_wide_compute_cap::id(),
    spl_token_v2_set_authority_fix::id(),
    merge_nonce_error_into_system_error::id(),
    disable_fees_sysvar::id(),
    stake_merge_with_unmatched_credits_observed::id(),
    curve25519_syscall_enabled::id(),
    curve25519_restrict_msm_length::id(),
    versioned_tx_message_enabled::id(),
    libsecp256k1_fail_on_bad_count2::id(),
    instructions_sysvar_owned_by_sysvar::id(),
    stake_program_advance_activating_credits_observed::id(),
    credits_auto_rewind::id(),
    demote_program_write_locks::id(),
    ed25519_program_enabled::id(),
    return_data_syscall_enabled::id(),
    reduce_required_deploy_balance::id(),
    sol_log_data_syscall_enabled::id(),
    stakes_remove_delegation_if_inactive::id(),
    do_support_realloc::id(),
    prevent_calling_precompiles_as_programs::id(),
    optimize_epoch_boundary_updates::id(),
    remove_native_loader::id(),
    send_to_tpu_vote_port::id(),
    requestable_heap_size::id(),
    disable_fee_calculator::id(),
    add_compute_budget_program::id(),
    nonce_must_be_writable::id(),
    spl_token_v3_3_0_release::id(),
    leave_nonce_on_success::id(),
    reject_empty_instruction_without_program::id(),
    fixed_memcpy_nonoverlapping_check::id(),
    reject_non_rent_exempt_vote_withdraws::id(),
    evict_invalid_stakes_cache_entries::id(),
    allow_votes_to_directly_update_vote_state::id(),
    max_tx_account_locks::id(),
    require_rent_exempt_accounts::id(),
    filter_votes_outside_slot_hashes::id(),
    update_syscall_base_costs::id(),
    stake_deactivate_delinquent_instruction::id(),
    vote_withdraw_authority_may_change_authorized_voter::id(),
    spl_associated_token_account_v1_0_4::id(),
    reject_vote_account_close_unless_zero_credit_epoch::id(),
    add_get_processed_sibling_instruction_syscall::id(),
    bank_transaction_count_fix::id(),
    disable_bpf_deprecated_load_instructions::id(),
    disable_bpf_unresolved_symbols_at_runtime::id(),
    record_instruction_in_transaction_context_push::id(),
    syscall_saturated_math::id(),
    check_physical_overlapping::id(),
    limit_secp256k1_recovery_id::id(),
    disable_deprecated_loader::id(),
    check_slice_translation_size::id(),
    stake_split_uses_rent_sysvar::id(),
    add_get_minimum_delegation_instruction_to_stake_program::id(),
    error_on_syscall_bpf_function_hash_collisions::id(),
    reject_callx_r10::id(),
    drop_redundant_turbine_path::id(),
    executables_incur_cpi_data_cost::id(),
    fix_recent_blockhashes::id(),
    update_rewards_from_cached_accounts::id(),
    spl_token_v3_4_0::id(),
    spl_associated_token_account_v1_1_0::id(),
    default_units_per_instruction::id(),
    stake_allow_zero_undelegated_amount::id(),
    require_static_program_ids_in_transaction::id(),
    add_set_compute_unit_price_ix::id(),
    disable_deploy_of_alloc_free_syscall::id(),
    include_account_index_in_rent_error::id(),
    add_shred_type_to_shred_seed::id(),
    warp_timestamp_with_a_vengeance::id(),
    separate_nonce_from_blockhash::id(),
    enable_durable_nonce::id(),
    vote_state_update_credit_per_dequeue::id(),
    quick_bail_on_panic::id(),
    nonce_must_be_authorized::id(),
    nonce_must_be_advanceable::id(),
    vote_authorize_with_seed::id(),
    preserve_rent_epoch_for_rent_exempt_accounts::id(),
    enable_bpf_loader_extend_program_ix::id(),
    enable_early_verification_of_account_modifications::id(),
    prevent_crediting_accounts_that_end_rent_paying::id(),
    cap_bpf_program_instruction_accounts::id(),
    use_default_units_in_fee_calculation::id(),
    compact_vote_state_updates::id(),
    incremental_snapshot_only_incremental_hash_calculation::id(),
    disable_cpi_setting_executable_and_rent_epoch::id(),
    on_load_preserve_rent_epoch_for_rent_exempt_accounts::id(),
    account_hash_ignore_slot::id(),
    set_exempt_rent_epoch_max::id(),
    relax_authority_signer_check_for_lookup_table_creation::id(),
    stop_sibling_instruction_search_at_parent::id(),
    vote_state_update_root_fix::id(),
    cap_accounts_data_allocations_per_transaction::id(),
    epoch_accounts_hash::id(),
    remove_deprecated_request_unit_ix::id(),
    disable_rehash_for_rent_epoch::id(),
    limit_max_instruction_trace_length::id(),
    check_syscall_outputs_do_not_overlap::id(),
    enable_bpf_loader_set_authority_checked_ix::id(),
    enable_alt_bn128_syscall::id(),
    simplify_alt_bn128_syscall_error_codes::id(),
    enable_alt_bn128_compression_syscall::id(),
    enable_program_redeployment_cooldown::id(),
    commission_updates_only_allowed_in_first_half_of_epoch::id(),
    enable_turbine_fanout_experiments::id(),
    move_serialized_len_ptr_in_cpi::id(),
    update_hashes_per_tick::id(),
    disable_builtin_loader_ownership_chains::id(),
    cap_transaction_accounts_data_size::id(),
    remove_congestion_multiplier_from_fee_calculation::id(),
    enable_request_heap_frame_ix::id(),
    prevent_rent_paying_rent_recipients::id(),
    delay_visibility_of_program_deployment::id(),
    add_set_tx_loaded_accounts_data_size_instruction::id(),
    switch_to_new_elf_parser::id(),
    round_up_heap_size::id(),
    remove_bpf_loader_incorrect_program_id::id(),
    native_programs_consume_cu::id(),
    simplify_writable_program_account_check::id(),
    stop_truncating_strings_in_syscalls::id(),
    clean_up_delegation_errors::id(),
    vote_state_add_vote_latency::id(),
    checked_arithmetic_in_fee_validation::id(),
    last_restart_slot_sysvar::id(),
    reduce_stake_warmup_cooldown::id(),
    enable_poseidon_syscall::id(),
    require_rent_exempt_split_destination::id(),
    better_error_codes_for_tx_lamport_check::id(),
    update_hashes_per_tick2::id(),
    update_hashes_per_tick3::id(),
    update_hashes_per_tick4::id(),
    update_hashes_per_tick5::id(),
    update_hashes_per_tick6::id(),
    validate_fee_collector_account::id(),
    drop_legacy_shreds::id(),
    consume_blockstore_duplicate_proofs::id(),
    index_erasure_conflict_duplicate_proofs::id(),
    merkle_conflict_duplicate_proofs::id(),
    disable_bpf_loader_instructions::id(),
    enable_gossip_duplicate_proof_ingestion::id(),
    deprecate_unused_legacy_vote_plumbing::id(),
];

/// Agave's currently active feature sets.
pub trait AgaveFeatures {
    fn mainnet_beta() -> Self;
    fn devnet() -> Self;
    fn testnet() -> Self;
    fn inactive() -> Self;
    fn all() -> Self;
}

impl AgaveFeatures for FeatureSet {
    fn mainnet_beta() -> Self {
        create_feature_set(AGAVE_FEATURES_MAINNET_BETA.iter(), |_| true)
    }

    fn devnet() -> Self {
        create_feature_set(AGAVE_FEATURES_DEVNET.iter(), |_| true)
    }

    fn testnet() -> Self {
        create_feature_set(AGAVE_FEATURES_TESTNET.iter(), |_| true)
    }

    fn inactive() -> Self {
        create_feature_set(AGAVE_FEATURES_INACTIVE.iter(), |_| true)
    }

    fn all() -> Self {
        let features = AGAVE_FEATURES_MAINNET_BETA
            .iter()
            .chain(AGAVE_FEATURES_DEVNET.iter())
            .chain(AGAVE_FEATURES_TESTNET.iter())
            .chain(AGAVE_FEATURES_INACTIVE.iter());
        create_feature_set(features, |_| true)
    }
}

impl AgaveFeatures for FixtureFeatureSet {
    fn mainnet_beta() -> Self {
        (&<FeatureSet as AgaveFeatures>::mainnet_beta()).into()
    }

    fn devnet() -> Self {
        (&<FeatureSet as AgaveFeatures>::devnet()).into()
    }

    fn testnet() -> Self {
        (&<FeatureSet as AgaveFeatures>::testnet()).into()
    }

    fn inactive() -> Self {
        (&<FeatureSet as AgaveFeatures>::inactive()).into()
    }

    fn all() -> Self {
        (&<FeatureSet as AgaveFeatures>::all()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_proto_feature_set() {
        let try_conversion = |feature_ids: &[Pubkey]| {
            let features = feature_ids
                .iter()
                .map(|id| u64::from_le_bytes(id.to_bytes()[..8].try_into().unwrap()))
                .collect::<Vec<_>>();
            let proto = proto::FeatureSet { features };
            let fixture = FixtureFeatureSet::from(proto);
            FeatureSet::from(fixture)
        };

        // Success
        let features = &[
            vote_state_add_vote_latency::id(),
            checked_arithmetic_in_fee_validation::id(),
            last_restart_slot_sysvar::id(),
            reduce_stake_warmup_cooldown::id(),
            enable_poseidon_syscall::id(),
            require_rent_exempt_split_destination::id(),
            better_error_codes_for_tx_lamport_check::id(),
        ];
        let feature_set = try_conversion(features);
        for feature in features {
            assert!(feature_set.is_active(feature));
        }

        // Not valid features (not in the list)
        let features = &[
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let feature_set = try_conversion(features);
        for feature in features {
            assert!(!feature_set.is_active(feature));
        }
    }
}
