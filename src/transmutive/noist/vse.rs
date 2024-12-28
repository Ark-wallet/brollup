use crate::{
    hash::Hash,
    into::{IntoPoint, IntoScalar},
    schnorr::{Bytes32, LiftScalar},
};
use secp::{MaybePoint, MaybeScalar, Point, Scalar};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn encrypting_key_secret(self_secret: [u8; 32], to_public: [u8; 32]) -> Option<[u8; 32]> {
    let self_secret = self_secret.into_scalar().ok()?;
    let to_public = to_public.into_point().ok()?;

    let shared_secret_point = self_secret * to_public;
    let shared_secret_point_bytes = shared_secret_point.serialize_uncompressed();
    let shared_secret_point_hash = (&shared_secret_point_bytes).hash();
    let shared_secret = match MaybeScalar::reduce_from(&shared_secret_point_hash) {
        MaybeScalar::Valid(scalar) => scalar.lift(),
        MaybeScalar::Zero => Scalar::reduce_from(&shared_secret_point_hash).lift(),
    };

    Some(shared_secret.serialize())
}

pub fn encrypting_key_public(self_secret: [u8; 32], to_public: [u8; 32]) -> Option<[u8; 32]> {
    let encrypting_key_secret = encrypting_key_secret(self_secret, to_public)?;
    encrypting_key_secret.secret_to_public()
}

pub fn encrypt(secret_to_encrypt: [u8; 32], encrypting_key_secret: [u8; 32]) -> Option<[u8; 32]> {
    let secret_to_encrypt = secret_to_encrypt.into_scalar().ok()?;
    let encrypting_key_secret = encrypting_key_secret.into_scalar().ok()?;

    match secret_to_encrypt + encrypting_key_secret {
        MaybeScalar::Valid(scalar) => Some(scalar.serialize()),
        MaybeScalar::Zero => None,
    }
}

pub fn decrypt(secret_to_decrypt: [u8; 32], encrypting_key_secret: [u8; 32]) -> Option<[u8; 32]> {
    let secret_to_decrypt = secret_to_decrypt.into_scalar().ok()?;
    let encrypting_key_secret = encrypting_key_secret.into_scalar().ok()?;

    match secret_to_decrypt - encrypting_key_secret {
        MaybeScalar::Valid(scalar) => Some(scalar.serialize()),
        MaybeScalar::Zero => None,
    }
}

pub fn verify(
    combined_scalar: [u8; 32],
    public_share_point: [u8; 33], // comperessed
    vse_public_key: [u8; 32],     // xonly
) -> bool {
    let combined_scalar = match Scalar::from_slice(&combined_scalar) {
        Ok(scalar) => scalar,
        Err(_) => return false,
    };

    let public_share_point = match Point::from_slice(&public_share_point) {
        Ok(point) => point,
        Err(_) => return false,
    };

    let vse_public_key = match Point::from_slice(&vse_public_key) {
        Ok(point) => point,
        Err(_) => return false,
    };
    let combined_point = combined_scalar.base_point_mul();

    combined_point
        == match public_share_point + vse_public_key {
            MaybePoint::Valid(point) => point,
            MaybePoint::Infinity => return false,
        }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct KeyMap {
    signer: [u8; 32],
    map: HashMap<[u8; 32], [u8; 32]>,
}

impl KeyMap {
    pub fn new(signer: [u8; 32]) -> KeyMap {
        KeyMap {
            signer,
            map: HashMap::<[u8; 32], [u8; 32]>::new(),
        }
    }

    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        match bincode::deserialize(&bytes) {
            Ok(keymap) => Some(keymap),
            Err(_) => None,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match bincode::serialize(&self) {
            Ok(bytes) => bytes,
            Err(_) => vec![],
        }
    }

    pub fn map(&self) -> HashMap<[u8; 32], [u8; 32]> {
        self.map.clone()
    }

    pub fn signer_key(&self) -> [u8; 32] {
        self.signer
    }

    pub fn insert(&mut self, signer_key: [u8; 32], vse_key: [u8; 32]) {
        if signer_key != self.signer {
            self.map.insert(signer_key, vse_key);
        }
    }

    pub fn map_list(&self) -> Vec<[u8; 32]> {
        let mut keys: Vec<[u8; 32]> = self.map.keys().cloned().collect();
        keys.sort();
        keys
    }

    pub fn full_list(&self) -> Vec<[u8; 32]> {
        let mut full_list = Vec::<[u8; 32]>::new();

        full_list.push(self.signer_key());
        full_list.extend(self.map_list());
        full_list.sort();

        full_list
    }

    pub fn is_complete(&self, expected_list: &Vec<[u8; 32]>) -> bool {
        let expected_list = {
            let mut expected_list_ = expected_list.clone();
            expected_list_.sort();
            expected_list_
        };

        let full_list = self.full_list();

        if full_list.len() == expected_list.len() {
            for (index, key) in full_list.iter().enumerate() {
                if key != &expected_list[index] {
                    return false;
                }
            }
            return true;
        }

        false
    }

    pub fn fill(&mut self, secret_key: [u8; 32], list: &Vec<[u8; 32]>) -> bool {
        for key in list {
            if key != &self.signer_key() {
                let vse_key = match encrypting_key_public(secret_key, *key) {
                    Some(key) => key,
                    None => return false,
                };
                self.insert(*key, vse_key);
            }
        }
        true
    }

    pub fn vse_key(&self, correspondant: [u8; 32]) -> Option<[u8; 32]> {
        Some(self.map.get(&correspondant)?.to_owned())
    }
}

pub struct Directory {
    signers: Vec<[u8; 32]>,
    vse_keys: Vec<KeyMap>,
}

impl Directory {
    pub fn new(signers: &Vec<[u8; 32]>) -> Directory {
        Directory {
            signers: signers.clone(),
            vse_keys: Vec::<KeyMap>::new(),
        }
    }

    pub fn signers(&self) -> Vec<[u8; 32]> {
        self.signers.clone()
    }

    pub fn insert(&mut self, map: KeyMap) -> bool {
        if self.signers.contains(&map.signer_key()) {
            if !self.vse_keys.contains(&map) {
                if map.is_complete(&self.signers()) {
                    self.vse_keys.push(map);
                }
                return true;
            }
        }
        false
    }

    pub fn map(&self, signer: [u8; 32]) -> Option<KeyMap> {
        for map in self.vse_keys.iter() {
            if map.signer_key() == signer {
                return Some(map.to_owned());
            }
        }

        None
    }

    pub fn is_complete(&self) -> bool {
        if self.vse_keys.len() != self.signers.len() {
            return false;
        }

        for map in self.vse_keys.iter() {
            if !map.is_complete(&self.signers()) {
                return false;
            }
        }

        true
    }

    pub fn validate(&self) -> bool {
        if !self.is_complete() {
            return false;
        }

        for signer in self.signers.iter() {
            let map = match self.map(signer.to_owned()) {
                Some(map) => map,
                None => return false,
            };
            let correspondants = map.map_list();

            for correspondant in correspondants.iter() {
                let vse_key_ = match self.vse_key(signer.to_owned(), correspondant.to_owned()) {
                    Some(key) => key,
                    None => return false,
                };
                let vse_key__ = match self.vse_key(correspondant.to_owned(), signer.to_owned()) {
                    Some(key) => key,
                    None => return false,
                };
                if vse_key_ != vse_key__ {
                    return false;
                }
            }
        }

        true
    }

    pub fn vse_key(&self, signer_1: [u8; 32], signer_2: [u8; 32]) -> Option<[u8; 32]> {
        for map in self.vse_keys.iter() {
            if map.signer_key() == signer_1 {
                if let Some(key) = map.vse_key(signer_2) {
                    return Some(key);
                }
            }
        }

        None
    }
}