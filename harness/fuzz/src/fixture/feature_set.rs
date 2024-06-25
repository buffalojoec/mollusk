//! A Solana runtime feature set, as represented in the Solana SDK.

use {
    super::proto,
    solana_sdk::{feature_set::*, pubkey::Pubkey},
    std::collections::HashSet,
};

// List of agave supported feature flags.
// As of `1.18.2`.
static AGAVE_FEATURES: &[Pubkey] = &[
    // Active on all clusters, but not cleaned up.
    pico_inflation::id(),
    warp_timestamp_again::id(),
    disable_fees_sysvar::id(),
    disable_deploy_of_alloc_free_syscall::id(),
    set_exempt_rent_epoch_max::id(),
    incremental_snapshot_only_incremental_hash_calculation::id(),
    relax_authority_signer_check_for_lookup_table_creation::id(),
    commission_updates_only_allowed_in_first_half_of_epoch::id(),
    enable_turbine_fanout_experiments::id(),
    update_hashes_per_tick::id(),
    reduce_stake_warmup_cooldown::id(),
    revise_turbine_epoch_stakes::id(),
    // Active on testnet & devnet.
    libsecp256k1_fail_on_bad_count2::id(),
    enable_bpf_loader_set_authority_checked_ix::id(),
    enable_alt_bn128_syscall::id(),
    switch_to_new_elf_parser::id(),
    vote_state_add_vote_latency::id(),
    require_rent_exempt_split_destination::id(),
    update_hashes_per_tick2::id(),
    update_hashes_per_tick3::id(),
    update_hashes_per_tick4::id(),
    update_hashes_per_tick5::id(),
    validate_fee_collector_account::id(),
    // Active on testnet.
    stake_raise_minimum_delegation_to_1_sol::id(),
    update_hashes_per_tick6::id(),
    // Active on devnet.
    blake3_syscall_enabled::id(),
    curve25519_syscall_enabled::id(),
    libsecp256k1_fail_on_bad_count::id(),
    reject_callx_r10::id(),
    increase_tx_account_lock_limit::id(),
    // Inactive on all clusters.
    zk_token_sdk_enabled::id(),
    enable_partitioned_epoch_reward::id(),
    stake_minimum_delegation_for_rewards::id(),
    stake_redelegate_instruction::id(),
    skip_rent_rewrites::id(),
    loosen_cpi_size_restriction::id(),
    disable_turbine_fanout_experiments::id(),
    enable_big_mod_exp_syscall::id(),
    apply_cost_tracker_during_replay::id(),
    include_loaded_accounts_data_size_in_fee_calculation::id(),
    bpf_account_data_direct_mapping::id(),
    last_restart_slot_sysvar::id(),
    enable_poseidon_syscall::id(),
    timely_vote_credits::id(),
    remaining_compute_units_syscall_enabled::id(),
    enable_program_runtime_v2_and_loader_v4::id(),
    enable_alt_bn128_compression_syscall::id(),
    disable_rent_fees_collection::id(),
    enable_zk_transfer_with_fee::id(),
    drop_legacy_shreds::id(),
    allow_commission_decrease_at_any_time::id(),
    consume_blockstore_duplicate_proofs::id(),
    index_erasure_conflict_duplicate_proofs::id(),
    merkle_conflict_duplicate_proofs::id(),
    enable_zk_proof_from_account::id(),
    curve25519_restrict_msm_length::id(),
    cost_model_requested_write_lock_cost::id(),
    enable_gossip_duplicate_proof_ingestion::id(),
    enable_chained_merkle_shreds::id(),
    // These two were force-activated, but the gate remains on the BPF Loader.
    disable_bpf_loader_instructions::id(),
];

impl From<proto::FeatureSet> for FeatureSet {
    fn from(input: proto::FeatureSet) -> Self {
        let mut feature_set = FeatureSet::default();
        let input_features: HashSet<u64> = input.features.into_iter().collect();

        for id in AGAVE_FEATURES.iter() {
            let discriminator = u64::from_le_bytes(id.to_bytes()[..8].try_into().unwrap());
            if input_features.contains(&discriminator) {
                feature_set.activate(id, 0);
            }
        }

        feature_set
    }
}

impl From<&FeatureSet> for proto::FeatureSet {
    fn from(input: &FeatureSet) -> Self {
        let features = input
            .active
            .keys()
            .map(|id| u64::from_le_bytes(id.to_bytes()[..8].try_into().unwrap()))
            .collect();

        proto::FeatureSet { features }
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

            FeatureSet::from(proto::FeatureSet { features })
        };

        // Success
        let features = &AGAVE_FEATURES[..10];
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
