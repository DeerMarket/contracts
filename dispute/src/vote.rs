/**
 * methods:
 *
 * - vote
 * - get_votes
 *
 */
use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum VoteType {
    Seller,
    Buyer,
    Draw,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Vote {
    pub voter: AccountId,
    pub dispute_id: u64,
    pub vote_type: VoteType,
    pub created_at: u64,
}

pub trait VoteInterface {
    fn vote(&mut self, dispute_id: u64, vote_type: VoteType);
    fn get_votes(&self, dispute_id: u64) -> Vec<Vote>;
}

#[near_bindgen]
impl VoteInterface for Contract {
    fn vote(&mut self, dispute_id: u64, vote_type: VoteType) {
        self.assert_whitelisted(&env::predecessor_account_id());

        let dispute = self
            .disputes_by_id
            .get(&dispute_id)
            .expect("ERR_DISPUTE_NOT_FOUND");

        require!(
            dispute.status == DisputeStatus::Voting,
            "ERR_DISPUTE_NOT_IN_VOTING"
        );

        require!(
            dispute.buyer_id != env::predecessor_account_id()
                && dispute.seller_id != env::predecessor_account_id(),
            "ERR_NOT_ALLOWED"
        );

        // check if the vote is not in the pending period
        require!(
            env::block_timestamp() > dispute.created_at + self.get_config().pending_period,
            "ERR_PENDING_PERIOD_NOT_ENDED"
        );

        let vote = Vote {
            voter: env::predecessor_account_id(),
            dispute_id,
            vote_type,
            created_at: env::block_timestamp(),
        };

        let mut votes = self
            .votes_by_dispute_id
            .get(&dispute_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKeys::Votes {
                        dispute_id_hash: dispute_id.try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });

        // check if the required votes have been reached
        if votes.len() >= dispute.required_votes {
            self.resolve_dispute(dispute_id);
            env::panic_str("ERR_DISPUTE_RESOLVED");
        }

        // check if the user has already voted
        votes.iter().for_each(|v| {
            assert_ne!(v.voter, vote.voter, "ERR_VOTER_ALREADY_VOTED_FOR_DISPUTE");
        });

        votes.insert(&vote);
        self.votes_by_dispute_id.insert(&dispute_id, &votes);

        // check if the required votes have been reached
        if votes.len() >= dispute.required_votes {
            self.resolve_dispute(dispute_id);
            return;
        }
    }

    fn get_votes(&self, dispute_id: u64) -> Vec<Vote> {
        let votes = self
            .votes_by_dispute_id
            .get(&dispute_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKeys::Votes {
                        dispute_id_hash: dispute_id.try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        votes.iter().collect()
    }
}
