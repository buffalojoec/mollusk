//! Errors possible for parsing fixtures.

use thiserror::Error;

/// Errors possible for parsing fixtures.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error, PartialEq)]
pub enum FixtureError {
    /// Invalid protobuf bytes provided.
    #[error("Invalid protobuf")]
    InvalidProtobuf(#[from] prost::DecodeError),
    /// A provided integer is out of range.
    #[error("Integer out of range")]
    IntegerOutOfRange,
    /// A provided byte array is invalid for a `u128`.
    #[error("Invalid u128 bytes")]
    InvalidU128Bytes,
    /// A provided byte array is invalid for a `Hash`.
    #[error("Invalid hash bytes")]
    InvalidHashBytes,
    /// A provided byte array is invalid for a `Pubkey`.
    #[error("Invalid public key bytes")]
    InvalidPubkeyBytes,
    /// An account index of an instruction account refers to an account that
    /// is not present in the input accounts list.
    #[error("Account missing")]
    AccountMissing,
    /// The input fixture is invalid.
    #[error("Invalid fixture input")]
    InvalidFixtureInput,
    /// The output fixture is invalid.
    #[error("Invalid fixture output")]
    InvalidFixtureOutput,
    /// The provided JSON fixture is invalid.
    #[error("Invalid JSON fixture")]
    InvalidJsonFixture,
}
