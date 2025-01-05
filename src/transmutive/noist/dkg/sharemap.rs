use crate::{
    hash::Hash,
    into::{IntoPoint, IntoScalar},
    noist::{secret::secret_share_gen, setup::setup::VSESetup, vse, vss},
    schnorr::{generate_secret, Sighash},
};
use secp::{MaybeScalar, Point, Scalar};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct DKGShareMap {
    signatory: Point,
    vss_commitments: Vec<Point>,
    shares: HashMap<Point, (Point, Scalar)>,
}

impl DKGShareMap {
    pub fn new(
        secret_key: [u8; 32],
        public_key: [u8; 32],
        signatories: &Vec<[u8; 32]>,
    ) -> Option<Self> {
        let self_point = public_key.into_point().ok()?;

        let polynomial_secret = {
            let mut preimage = Vec::<u8>::new();
            preimage.extend(secret_key);
            preimage.extend(generate_secret());
            preimage
                .hash(Some(crate::hash::HashTag::SecretKey))
                .into_scalar()
                .ok()?
        };

        let num_signatories = signatories.len() as u8;

        if num_signatories < 3 {
            return None;
        }

        let threshold = (num_signatories / 2) + 1;

        let (secret_shares, vss_points) =
            secret_share_gen(polynomial_secret, num_signatories, threshold).ok()?;

        let mut vss_commitments = Vec::<Point>::new();
        {
            for vss_point in vss_points {
                vss_commitments.push(vss_point);
            }
        }

        let mut shares = HashMap::<Point, (Point, Scalar)>::new();

        {
            let mut signatories = signatories.clone();
            signatories.sort();

            for (index, signatory) in signatories.iter().enumerate() {
                let signatory_point = signatory.into_point().ok()?;
                let self_secret_scalar = secret_key.into_scalar().ok()?;

                let secret_share = secret_shares[index].1;
                let public_share = secret_share.base_point_mul();

                let secret_share_enc = {
                    if signatory_point == self_point {
                        Scalar::one()
                    } else {
                        let encrypting_key_secret =
                            vse::encrypting_key_secret(self_secret_scalar, signatory_point);
                        vse::encrypt(secret_share, encrypting_key_secret).ok()?
                    }
                };

                shares.insert(signatory_point, (public_share, secret_share_enc));
            }
        }

        Some(DKGShareMap {
            signatory: self_point,
            vss_commitments,
            shares,
        })
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

    pub fn signatory(&self) -> Point {
        self.signatory.clone()
    }

    pub fn vss_commitments(&self) -> Vec<Point> {
        self.vss_commitments.clone()
    }

    pub fn shares(&self) -> HashMap<Point, (Point, Scalar)> {
        self.shares.clone()
    }

    pub fn ordered_shares(&self) -> Vec<(Point, (Point, Scalar))> {
        let mut maps: Vec<(Point, (Point, Scalar))> = self.shares().into_iter().collect();
        maps.sort_by(|a, b| a.0.cmp(&b.0));
        maps
    }

    pub fn share_by_key(&self, key: [u8; 32]) -> Option<(Point, Scalar)> {
        let key_point = key.into_point().ok()?;
        let share = self.shares.get(&key_point)?;
        Some(share.to_owned())
    }

    pub fn share_by_index(&self, index: u8) -> Option<(Point, Scalar)> {
        let ordered_shares = self.ordered_shares();
        let share = ordered_shares.get(index as usize)?;
        Some(share.1)
    }

    pub fn constant_point(&self) -> Option<Point> {
        let constant_point = self.vss_commitments.get(0)?;
        Some(constant_point.to_owned())
    }

    pub fn is_complete(&self, signatories: &Vec<[u8; 32]>) -> bool {
        let mut signatories = signatories.clone();
        signatories.sort();

        if signatories.len() != self.shares.len() {
            return false;
        }

        for (index, (signatory, _)) in self.ordered_shares().iter().enumerate() {
            if signatory.serialize_xonly() != signatories[index] {
                return false;
            }
        }

        true
    }

    pub fn vss_verify(&self) -> bool {
        let mut vss_commitments = Vec::<Point>::new();

        for vss_commitment in self.vss_commitments.iter() {
            vss_commitments.push(vss_commitment.to_owned());
        }

        for (index, (_, (pubshare, _))) in self.ordered_shares().iter().enumerate() {
            let index_scalar = match MaybeScalar::from((index + 1) as u128) {
                MaybeScalar::Valid(scalar) => scalar,
                MaybeScalar::Zero => return false,
            };

            let share_i = (index_scalar, pubshare.to_owned());

            if !vss::vss_verify_point(share_i, &vss_commitments) {
                return false;
            }
        }

        true
    }

    pub fn vse_verify(&self, setup: &VSESetup) -> bool {
        for (key, (pubshare, encsec)) in self.shares.iter() {
            if self.signatory == key.to_owned() {
                if encsec != &Scalar::one() {
                    return false;
                }
            } else {
                let vse_point = match setup.vse_point(self.signatory, key.to_owned()) {
                    Some(vse_key) => vse_key,
                    None => return false,
                };

                if !vse::verify(encsec.to_owned(), pubshare.to_owned(), vse_point) {
                    return false;
                }
            }
        }

        true
    }

    pub fn print(&self) {
        println!("VSS Commitments :");
        for (index, vss_commitment) in self.vss_commitments.iter().enumerate() {
            let str = match index {
                0 => format!(
                    "#0 -> {} (Constant Point)",
                    hex::encode(vss_commitment.serialize())
                ),
                _ => format!("#{} -> {}", index, hex::encode(vss_commitment.serialize())),
            };
            println!("{}", str);
        }

        println!("");

        println!("Shares :");
        for (index, (key, (pubshare, encsec))) in self.ordered_shares().iter().enumerate() {
            println!("#{} {}", index, hex::encode(key.serialize_xonly()));
            println!("   pubshare: {}", hex::encode(pubshare.serialize()));
            println!("   encsec: {}\n", hex::encode(encsec.serialize()));
        }
    }
}

impl Sighash for DKGShareMap {
    fn sighash(&self) -> [u8; 32] {
        let mut preimage = Vec::<u8>::new();
        preimage.extend(self.signatory.serialize_xonly());

        for vss_commitment in self.vss_commitments.iter() {
            preimage.extend(vss_commitment.serialize_uncompressed());
        }

        for share in self.shares.iter() {
            preimage.extend(share.0.serialize_xonly());
            preimage.extend(share.1 .0.serialize_uncompressed());
            preimage.extend(share.1 .1.serialize());
        }

        preimage.hash(Some(crate::hash::HashTag::SighashAuthenticable))
    }
}