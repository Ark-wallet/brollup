use crate::{valtype::account::Account, BLAMING_DIRECTORY};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Unix timestamp which a particular account is banned until.
type BlameCounter = u16;
type BlacklistedUntil = u64;

// Initial blame window period is 10 seconds.
const INITIAL_BLAME_SECS_WINDOW: u64 = 5;

/// Directory for the coordinator to manage account blacklists.
pub struct BlamingDirectory {
    // In-memory list.
    list: HashMap<Account, (BlameCounter, BlacklistedUntil)>,
    // In-storage db.
    db: sled::Db,
}

impl BlamingDirectory {
    pub fn new() -> Option<BLAMING_DIRECTORY> {
        let db = sled::open("db/blamingdir").ok()?;

        let mut list = HashMap::<Account, (BlameCounter, BlacklistedUntil)>::new();

        for lookup in db.iter() {
            if let Ok((key, val)) = lookup {
                let account: Account = serde_json::from_slice(&key).ok()?;

                let (blame_counter, blacklisted_until) = serde_json::from_slice(&val).ok()?;

                list.insert(account, (blame_counter, blacklisted_until));
            }
        }

        let blaming_dir = BlamingDirectory { list, db };

        Some(Arc::new(Mutex::new(blaming_dir)))
    }

    /// Inserts or updates blaming records for a given accounts.
    fn update(
        &mut self,
        account: Account,
        blame_counter: BlameCounter,
        blacklisted_until: BlacklistedUntil,
    ) {
        // Update in-memory
        self.list
            .insert(account, (blame_counter, blacklisted_until));

        // Update in-storage
        {
            let key_serialized = account.serialize();

            let value_serialized = match serde_json::to_vec(&(blame_counter, blacklisted_until)) {
                Ok(bytes) => bytes,
                Err(_) => vec![],
            };

            let _ = self.db.insert(key_serialized, value_serialized);
        }
    }

    /// Blames an account.
    pub fn blame(&mut self, account: Account) {
        match self.list.get(&account) {
            Some((counter, until)) => {
                let mut blame_counter: u16 = counter.to_owned();
                let mut blacklisted_until: u64 = until.to_owned();

                if blame_counter < u16::MAX {
                    blame_counter = blame_counter + 1;
                    blacklisted_until =
                        current_unix_timestamp() + (2 as u64).pow(blame_counter as u32);
                } else if blame_counter == u16::MAX {
                    // Permaban
                    blacklisted_until = u64::MAX;
                }

                self.update(account, blame_counter, blacklisted_until);
            }
            None => {
                let blame_counter: u16 = 1;
                let blamed_until: u64 = current_unix_timestamp() + INITIAL_BLAME_SECS_WINDOW;

                self.update(account, blame_counter, blamed_until);
            }
        };
    }

    /// Checks whether an account is blacklisted. Returns the timestamp if any.
    pub fn check_blacklist(&self, account: Account) -> Option<u64> {
        match self.list.get(&account) {
            Some((_, blacklisted_until)) => {
                match blacklisted_until.to_owned() > current_unix_timestamp() {
                    true => Some(blacklisted_until.to_owned()),
                    false => None,
                }
            }
            None => None,
        }
    }
}

fn current_unix_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
