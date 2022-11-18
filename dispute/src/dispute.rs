/**
 *
 * methods:
 * - create_dispute
 * - resolve_dispute
 * - get_dispute
 * - get_disputes
 * - get_disputes_by_account_id
 *
 */
use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum DisputeStatus {
    // waiting for voter to vote
    Voting,
    // seller won
    SellerWon,
    // buyer won
    BuyerWon,
    // draw, they split the loss
    Draw,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Dispute {
    pub id: u64,
    pub disputer: AccountId,
    pub fee: Balance,
    pub required_votes: u64,
    pub status: DisputeStatus,
    pub created_at: u64,
    pub resolved_at: u64,
    //
    pub store_id: AccountId,
    pub item_id: String,
    pub order_id: String,
    pub buyer_id: AccountId,
    pub seller_id: AccountId,
    pub description: String,
}

impl Dispute {
    pub fn new(
        id: u64,
        disputer: AccountId,
        fee: Balance,
        required_votes: u64,
        store_id: AccountId,
        item_id: String,
        order_id: String,
        buyer_id: AccountId,
        seller_id: AccountId,
        description: String,
    ) -> Self {
        Self {
            id,
            disputer,
            fee,
            required_votes,
            status: DisputeStatus::Voting,
            created_at: env::block_timestamp(),
            resolved_at: 0,
            store_id,
            item_id,
            order_id,
            buyer_id,
            seller_id,
            description,
        }
    }
}

pub trait DisputeInterface {
    fn create_dispute(
        &mut self,
        store_id: AccountId,
        item_id: String,
        order_id: String,
        buyer_id: AccountId,
        seller_id: AccountId,
        description: String,
    ) -> u64;
    fn get_dispute(&self, dispute_id: u64) -> Option<Dispute>;
    fn get_disputes(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Dispute>;
    fn get_disputes_by_account_id(&self, account_id: AccountId) -> Vec<Dispute>;
    fn resolve_dispute(&mut self, dispute_id: u64);
}

#[near_bindgen]
impl DisputeInterface for Contract {
    #[payable]
    fn create_dispute(
        &mut self,
        store_id: AccountId,
        item_id: String,
        order_id: String,
        buyer_id: AccountId,
        seller_id: AccountId,
        description: String,
    ) -> u64 {
        let config = self.get_config();

        // assert enough deposit
        require!(
            env::attached_deposit() >= config.min_fee,
            "Not enough deposit to create dispute"
        );

        // calculate required votes
        let required_votes = (env::attached_deposit() / ONE_NEAR) * config.votes_per_near as u128;
        let required_votes = if required_votes > config.max_votes as u128 {
            config.max_votes as u128
        } else if required_votes < 1 as u128 {
            1 as u128
        } else {
            required_votes
        };

        let id = self.disputes_by_id.len() as u64;
        let dispute = Dispute::new(
            id,
            env::signer_account_id(),
            env::attached_deposit(),
            required_votes as u64,
            store_id,
            item_id,
            order_id,
            buyer_id.clone(),
            seller_id.clone(),
            description,
        );
        self.disputes_by_id.insert(&id, &dispute);

        let mut seller_disputes =
            self.disputes_by_account_id
                .get(&seller_id)
                .unwrap_or_else(|| {
                    UnorderedSet::new(
                        StorageKeys::DisputesByAccountId {
                            account_id_hash: seller_id.try_to_vec().unwrap(),
                        }
                        .try_to_vec()
                        .unwrap(),
                    )
                });
        seller_disputes.insert(&id);

        let mut buyer_disputes = self
            .disputes_by_account_id
            .get(&buyer_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKeys::DisputesByAccountId {
                        account_id_hash: buyer_id.try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        buyer_disputes.insert(&id);

        id
    }

    fn get_dispute(&self, dispute_id: u64) -> Option<Dispute> {
        self.disputes_by_id.get(&dispute_id)
    }

    fn get_disputes(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Dispute> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(0);

        //iterate through each item using an iterator
        self.disputes_by_id
            .keys()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(50) as usize)
            //we'll map the item IDs which are strings into Json Tokens
            .map(|item_id| self.get_dispute(item_id).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    fn get_disputes_by_account_id(&self, account_id: AccountId) -> Vec<Dispute> {
        let dispute_ids = self
            .disputes_by_account_id
            .get(&account_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKeys::DisputesByAccountId {
                        account_id_hash: account_id.try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });

        dispute_ids
            .iter()
            .map(|dispute_id| self.disputes_by_id.get(&dispute_id).unwrap())
            .collect()
    }

    #[private]
    fn resolve_dispute(&mut self, dispute_id: u64) {
        let dispute = self.disputes_by_id.get(&dispute_id).unwrap();
        let votes = self.votes_by_dispute_id.get(&dispute_id).unwrap();

        let mut status = DisputeStatus::Draw;

        let mut seller_won = 0;
        let mut buyer_won = 0;

        for vote in votes.iter() {
            if vote.vote_type == VoteType::Seller {
                seller_won += 1;
            } else if vote.vote_type == VoteType::Buyer {
                buyer_won += 1;
            }
        }

        if seller_won > buyer_won {
            status = DisputeStatus::SellerWon;
        } else if buyer_won > seller_won {
            status = DisputeStatus::BuyerWon;
        }

        let mut dispute = dispute.clone();

        dispute.status = status;
        dispute.resolved_at = env::block_timestamp();

        self.disputes_by_id.insert(&dispute_id, &dispute);

        // distribute fee prize to voters
        let total_votes = votes.len();
        let one_share = dispute.fee / total_votes as u128;

        for vote in votes.iter() {
            Promise::new(vote.voter.clone()).transfer(one_share);
        }

        // Call the store contract to resolve the dispute
        Promise::new(dispute.store_id.clone()).function_call(
            "dispute_resolve".to_string(),
            to_vec(&json!({
                "order_id": dispute_id.to_string(),
                "resolution": dispute.status,
            }))
            .unwrap(),
            0,
            env::prepaid_gas() - env::used_gas() - Gas::ONE_TERA * 10,
        );
    }
}
