/**
 * Dispute resolution system.
 */
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{json, to_vec};
use near_sdk::{
    env, near_bindgen, require, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise,
    ONE_NEAR,
};

mod dispute;
mod evidence;
mod security;
mod vote;

use crate::dispute::*;
use crate::evidence::*;
use crate::security::*;
use crate::vote::*;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Config,
    Whitelist,
    Disputes,
    DisputesByAccountId { account_id_hash: Vec<u8> },
    Votes { dispute_id_hash: Vec<u8> },
    Evidence { dispute_id_hash: Vec<u8> },
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Config
    pub config: LazyOption<Config>,

    /// whitelist
    pub whitelist: UnorderedSet<AccountId>,

    /// Disputes
    pub disputes_by_id: UnorderedMap<u64, Dispute>,
    pub disputes_by_account_id: LookupMap<AccountId, UnorderedSet<u64>>,

    /// Votes
    pub votes_by_dispute_id: LookupMap<u64, UnorderedSet<Vote>>,

    /// Evidence
    pub evidence_by_dispute_id: LookupMap<u64, UnorderedSet<Evidence>>,
}

// TODO: Secure against contract funds being drained by malicious calls
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            config: LazyOption::new(StorageKeys::Config, Some(&Config::default())),
            whitelist: UnorderedSet::new(StorageKeys::Whitelist),
            disputes_by_id: UnorderedMap::new(StorageKeys::Disputes),
            disputes_by_account_id: LookupMap::new(StorageKeys::DisputesByAccountId {
                account_id_hash: vec![],
            }),
            votes_by_dispute_id: LookupMap::new(StorageKeys::Votes {
                dispute_id_hash: vec![],
            }),
            evidence_by_dispute_id: LookupMap::new(StorageKeys::Evidence {
                dispute_id_hash: vec![],
            }),
        }
    }
}
