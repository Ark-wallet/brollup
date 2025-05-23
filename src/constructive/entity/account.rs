use crate::{
    constructive::cpe::{
        cpe::CompactPayloadEncoding,
        decode_error::{entity_error::AccountCPEDecodingError, error::CPEDecodingError},
    },
    constructive::valtype::short_val::ShortVal,
    inscriptive::registery::account_registery::ACCOUNT_REGISTERY,
};
use async_trait::async_trait;
use bit_vec::BitVec;
use secp::Point;
use serde::{Deserialize, Serialize};

/// Represents an account; a user of the system.
#[derive(Clone, Copy, Serialize, Deserialize, Hash, Debug)]
pub struct Account {
    key: Point,
    registery_index: Option<ShortVal>,
    rank: Option<ShortVal>,
}

impl Account {
    /// Creates a new account.
    pub fn new(key: Point, registery_index: Option<u32>, rank: Option<u32>) -> Option<Account> {
        let is_odd: bool = key.parity().into();

        if is_odd {
            return None;
        }

        // Convert the registery index to a ShortVal.
        let registery_index = match registery_index {
            Some(index) => Some(ShortVal::new(index)),
            None => None,
        };

        // Convert the rank to a ShortVal.
        let rank = match rank {
            Some(rank) => Some(ShortVal::new(rank)),
            None => None,
        };

        let account = Account {
            key,
            registery_index,
            rank,
        };

        Some(account)
    }

    /// Returns the registery index of the account.
    pub fn registery_index(&self) -> Option<u32> {
        self.registery_index.map(|index| index.value())
    }

    /// Sets the registery index of the account.
    pub fn set_registery_index(&mut self, registery_index: u32) {
        self.registery_index = Some(ShortVal::new(registery_index));
    }

    /// Returns the rank (if set).
    pub fn rank(&self) -> Option<u32> {
        self.rank.map(|rank| rank.value())
    }

    /// Sets the rank index.
    pub fn set_rank(&mut self, rank: Option<u32>) {
        self.rank = rank.map(|rank| ShortVal::new(rank));
    }

    /// Returns the key of the account.
    pub fn key(&self) -> Point {
        self.key
    }

    /// Returns true if the key is odd.
    pub fn is_odd_key(&self) -> bool {
        self.key.parity().into()
    }

    /// Serializes the account.
    pub fn serialize(&self) -> Vec<u8> {
        match serde_json::to_vec(self) {
            Ok(bytes) => bytes,
            Err(_) => vec![],
        }
    }

    /// Compact payload decoding for `Account`.
    /// Decodes an `Account` from a bit stream.  
    pub async fn decode_cpe<'a>(
        bit_stream: &mut bit_vec::Iter<'a>,
        account_registery: &ACCOUNT_REGISTERY,
    ) -> Result<Account, CPEDecodingError> {
        // Decode the rank value.
        let rank = ShortVal::decode_cpe(bit_stream)
            .map_err(|_| {
                CPEDecodingError::AccountCPEDecodingError(
                    AccountCPEDecodingError::FailedToDecodeRank,
                )
            })?
            .value();

        // Match the rank value to determine if the account is registered or not.
        // If rank is zero, then we interpret this as an unregistered account, otherwise it is a registered account.
        match rank {
            0 => {
                // Unregistered account.

                // Collect exactly 256 bits for the public key.
                let public_key_bits: BitVec = bit_stream.by_ref().take(256).collect();

                // Ensure the collected bits are the correct length.
                if public_key_bits.len() != 256 {
                    return Err(CPEDecodingError::AccountCPEDecodingError(
                        AccountCPEDecodingError::FailedToColletKeyBits,
                    ));
                }

                // Convert public key bits to an even public key bytes.
                let mut public_key_bytes = vec![0x02];
                public_key_bytes.extend(public_key_bits.to_bytes());

                // Construct the public key.
                let public_key = Point::from_slice(&public_key_bytes).map_err(|_| {
                    CPEDecodingError::AccountCPEDecodingError(
                        AccountCPEDecodingError::FailedToConstructKey,
                    )
                })?;

                // Check if the key is already registered.
                let is_registered = {
                    let _account_registery = account_registery.lock().await;
                    _account_registery.is_registered(public_key)
                };

                // If the key is already registered, return an error.
                if is_registered {
                    return Err(CPEDecodingError::AccountCPEDecodingError(
                        AccountCPEDecodingError::AccountKeyAlreadyRegistered(
                            public_key.serialize_xonly(),
                        ),
                    ));
                }

                // Construct the unregistered account.
                let account = Account {
                    key: public_key,
                    registery_index: None,
                    rank: None,
                };

                // Return the `Account`.
                return Ok(account);
            }
            _ => {
                // Registered account.

                // Retrieve the account given rank value.
                let account = {
                    let _account_registery = account_registery.lock().await;
                    _account_registery.account_by_rank(rank).ok_or(
                        CPEDecodingError::AccountCPEDecodingError(
                            AccountCPEDecodingError::FailedToLocateAccountGivenRank(rank),
                        ),
                    )?
                };

                // Return the `Account`.
                return Ok(account);
            }
        }
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Account {}

#[async_trait]
impl CompactPayloadEncoding for Account {
    fn encode_cpe(&self) -> Option<BitVec> {
        let mut bits = BitVec::new();

        // Match on the rank value.
        match self.rank {
            // If the rank is set, then we interpret this as a registered account.
            Some(rank) => {
                // Extend rank bits.
                bits.extend(rank.encode_cpe()?);
            }
            // If the rank is not set, then we interpret this as an unregistered account.
            None => {
                // Extend with the rank value zero.
                bits.extend(ShortVal::new(0).encode_cpe()?);

                // Public key bits.
                let public_key_bits = BitVec::from_bytes(&self.key.serialize_xonly());

                // Extend public key bits.
                bits.extend(public_key_bits);
            }
        };

        Some(bits)
    }
}
