use super::{
    account_registery::{AccountRegistery, ACCOUNT_REGISTERY},
    contract_registery::{ContractRegistery, CONTRACT_REGISTERY},
};
use crate::Network;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type REGISTERY = Arc<Mutex<Registery>>;

/// Directory for the account registeries.
pub struct Registery {
    account_registery: ACCOUNT_REGISTERY,
    contract_registery: CONTRACT_REGISTERY,
}

impl Registery {
    pub fn new(network: Network) -> Option<REGISTERY> {
        let account_registery = AccountRegistery::new(network)?;
        let contract_registery = ContractRegistery::new(network)?;
        let registery = Registery {
            account_registery,
            contract_registery,
        };

        Some(Arc::new(Mutex::new(registery)))
    }

    pub fn account_registery(&self) -> ACCOUNT_REGISTERY {
        Arc::clone(&self.account_registery)
    }

    pub fn contract_registery(&self) -> CONTRACT_REGISTERY {
        Arc::clone(&self.contract_registery)
    }
}
