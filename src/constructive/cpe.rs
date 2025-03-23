use async_trait::async_trait;
use bit_vec::BitVec;
use serde::{Deserialize, Serialize};

/// Trait for encoding structs for compact Bitcoin-DA storage.
#[async_trait]
pub trait CompactPayloadEncoding {
    /// Encode the struct into a bitvec.
    fn encode_cpe(&self) -> BitVec;
}

/// Compact payload decoding is implemented individually for each struct that implements `CompactPayloadEncoding`, rather than using a trait.
/// Refer to the CPE decoding error types listed below.

/// Error type for `AtomicVal` CPE decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtomicValCPEDecodingError {
    // Bit stream iteration error.
    BitStreamIteratorError,
}

/// Error type for `ShortVal` CPE decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShortValCPEDecodingError {
    // Bit stream iteration error.
    BitStreamIteratorError,
    // Short value conversion error.
    ShortValConversionError,
}

/// Error type for `LongVal` CPE decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LongValCPEDecodingError {
    // Bit stream iteration error.
    BitStreamIteratorError,
    // Long value conversion error.
    LongValConversionError,
}

/// Error type for `Account` CPE decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountCPEDecodingError {
    // Failed to iterate over the bit stream to check if the account is registered.
    FailedToIterateIsRegisteredBit,
    // Failed to decode the registery index.
    FailedToDecodeRegisteryIndex,
    // Unable to locate the account key from the registery index.
    UnableToLocateAccountKeyGivenIndex(u32),
    // Unable to construct a new key to be registered.
    UnableToConstructNewKey,
    // Account key is already registered.
    AccountKeyAlreadyRegistered([u8; 32]),
}

/// Error type for `Contract` CPE decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractCPEDecodingError {
    // Failed to decode the registery index.
    FailedToDecodeRegisteryIndex,
    // Unable to locate the contract ID from the registery index.
    UnableToLocateContractIdGivenIndex(u32),
}

/// Error type for `Liftup` CPE decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiftupCPEDecodingError {
    // Unable to find a matching `Lift` at the given transaction input iterator position.
    NoLiftAtInputIter(u32),
    // Unable to re-construct `Lift` at the given transaction input iterator position.
    LiftReconstructionErrAtInputIter(u32),
    // Unable to find a matching `Lift` at the given transaction input iterator position.
    NoMatchingLiftAtInputIter(u32),
}

/// Error type for compact payload decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CPEDecodingError {
    // Atomic value CPE decoding error.
    AtomicValCPEDecodingError(AtomicValCPEDecodingError),
    // Short value CPE decoding error.
    ShortValCPEDecodingError(ShortValCPEDecodingError),
    // Long value CPE decoding error.
    LongValCPEDecodingError(LongValCPEDecodingError),
    // Account CPE decoding error.
    AccountCPEDecodingError(AccountCPEDecodingError),
    // Contract CPE decoding error.
    ContractCPEDecodingError(ContractCPEDecodingError),
    // Liftup CPE decoding error.
    LiftupCPEDecodingError(LiftupCPEDecodingError),
}
