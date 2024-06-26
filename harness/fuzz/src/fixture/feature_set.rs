//! A Solana runtime feature set, as represented in the Solana SDK.

use {
    super::proto,
    serde::{Deserialize, Serialize},
    solana_sdk::feature_set::*,
};

#[derive(Debug, Deserialize, Serialize)]
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

impl From<&FixtureFeatureSet> for proto::FeatureSet {
    fn from(input: &FixtureFeatureSet) -> Self {
        proto::FeatureSet {
            features: input.features.clone(),
        }
    }
}

impl From<FixtureFeatureSet> for FeatureSet {
    fn from(input: FixtureFeatureSet) -> Self {
        let mut feature_set = FeatureSet::default();
        for id in std::mem::take(&mut feature_set.inactive).iter() {
            let discriminator = u64::from_le_bytes(id.to_bytes()[..8].try_into().unwrap());
            if input.features.contains(&discriminator) {
                feature_set.activate(&id, 0);
            }
        }
        feature_set
    }
}

impl From<&FeatureSet> for FixtureFeatureSet {
    fn from(input: &FeatureSet) -> Self {
        let features = input
            .active
            .iter()
            .map(|(id, _)| u64::from_le_bytes(id.to_bytes()[..8].try_into().unwrap()))
            .collect();
        Self { features }
    }
}
