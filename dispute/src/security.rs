/**
 * Contract security & configuration
 *
 * - whitelist
 * - ban
 * - assert_owner
 * - assert_whitelisted
 * - get_config
 */
use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub owner_id: AccountId,

    // the stake required to be able to vote
    pub participation_stake: Balance,

    // time to wait before users can vote
    pub pending_period: u64,

    // time users have to vote before the dispute is closed
    pub voting_period: u64,

    // minimum fee required to create a dispute
    pub min_fee: Balance,

    // votes required to resolve a dispute for each 1 NEAR in fees
    pub votes_per_near: u64,

    // maximum votes allowed for a dispute despite the amount of fees
    pub max_votes: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            owner_id: env::predecessor_account_id(),
            participation_stake: 10000000000000000000000, // 10 NEAR
            // pending_period: 86400,                        // 1 day
            pending_period: 180,           // 3 minute
            voting_period: 604800,         // 7 days
            min_fee: 20000000000000000000, // 0.2 NEAR
            votes_per_near: 1,             // 1 vote per 1 NEAR
            max_votes: 1000,               // 1000 votes
        }
    }
}

pub trait SecurityProvider {
    fn assert_owner(&self);
    fn assert_whitelisted(&self, account_id: &AccountId);
    fn whitelist(&mut self, account_id: AccountId);
    fn ban(&mut self, account_id: AccountId);
    fn get_config(&self) -> Config;
}

#[near_bindgen]
impl SecurityProvider for Contract {
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.config.get().unwrap().owner_id,
            "Only owner can call this method"
        );
    }

    fn assert_whitelisted(&self, account_id: &AccountId) {
        assert!(
            self.whitelist.contains(account_id),
            "Account is not whitelisted"
        );
    }

    // anyone can whitelist himself currently but in the future should be only owner allowed
    fn whitelist(&mut self, account_id: AccountId) {
        // self.assert_owner();
        self.whitelist.insert(&account_id);
    }

    fn ban(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.whitelist.remove(&account_id);
    }

    fn get_config(&self) -> Config {
        self.config.get().unwrap()
    }
}
