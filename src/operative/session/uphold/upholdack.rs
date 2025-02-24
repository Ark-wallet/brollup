use crate::{txo::lift::Lift, valtype::account::Account};
use secp::Scalar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// `CSessionUpholdAck` contains the full aggreagte covenant signatures and is returned by the coordinator
/// to the msg.senders upon receiving `NSessionUphold` if all `NSessionUphold`s were sucesfully collected.
/// Otherwise, the coordinator responds with `CSessionUpholdError`.
#[derive(Clone, Serialize, Deserialize)]
pub struct CSessionUpholdAck {
    // Account
    account: Account,
    // Payload auth
    payload_auth_agg_sig: Scalar,
    // VTXO projector
    vtxo_projector_agg_sig: Option<Scalar>,
    // Connector projector
    connector_projector_agg_sig: Option<Scalar>,
    // ZKP contingent
    zkp_contingent_agg_sig: Option<Scalar>,
    // Lift txos
    lift_prevtxo_agg_sigs: HashMap<Lift, Scalar>,
    // Connectors
    connector_txo_agg_sigs: Vec<Scalar>,
    // TODO: forfeiture locations..
}

impl CSessionUpholdAck {
    pub fn new(
        account: Account,
        payload_auth_agg_sig: Scalar,
        vtxo_projector_agg_sig: Option<Scalar>,
        connector_projector_agg_sig: Option<Scalar>,
        zkp_contingent_agg_sig: Option<Scalar>,
        lift_prevtxo_agg_sigs: HashMap<Lift, Scalar>,
        connector_txo_agg_sigs: Vec<Scalar>,
    ) -> CSessionUpholdAck {
        CSessionUpholdAck {
            account,
            payload_auth_agg_sig,
            vtxo_projector_agg_sig,
            connector_projector_agg_sig,
            zkp_contingent_agg_sig,
            lift_prevtxo_agg_sigs,
            connector_txo_agg_sigs,
        }
    }

    pub fn account(&self) -> Account {
        self.account.clone()
    }

    pub fn payload_auth_agg_sig(&self) -> Scalar {
        self.payload_auth_agg_sig.clone()
    }

    pub fn vtxo_projector_agg_sig(&self) -> Option<Scalar> {
        self.vtxo_projector_agg_sig.clone()
    }

    pub fn connector_projector_agg_sig(&self) -> Option<Scalar> {
        self.connector_projector_agg_sig.clone()
    }

    pub fn zkp_contingent_agg_sig(&self) -> Option<Scalar> {
        self.zkp_contingent_agg_sig.clone()
    }

    pub fn lift_prevtxo_agg_sigs(&self) -> HashMap<Lift, Scalar> {
        self.lift_prevtxo_agg_sigs.clone()
    }

    pub fn connector_txo_agg_sigs(&self) -> Vec<Scalar> {
        self.connector_txo_agg_sigs.clone()
    }
}
