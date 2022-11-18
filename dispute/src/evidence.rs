/**
 * methods:
 *
 * - add_evidence
 * - get_evidence
 *
 */
use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum EvidenceType {
    Seller,
    Buyer,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub dispute_id: u64,
    pub description: String,
    pub link: String,
    pub created_at: u64,
}

pub trait EvidenceInterface {
    fn add_evidence(&mut self, dispute_id: u64, description: String, link: String);
    fn get_evidence(&self, dispute_id: u64) -> Vec<Evidence>;
}

#[near_bindgen]
impl EvidenceInterface for Contract {
    fn add_evidence(&mut self, dispute_id: u64, description: String, link: String) {
        let dispute = self
            .disputes_by_id
            .get(&dispute_id)
            .expect("ERR_DISPUTE_NOT_FOUND");

        // check if the dispute is in voting status
        assert_eq!(
            dispute.status,
            DisputeStatus::Voting,
            "ERR_DISPUTE_NOT_IN_VOTING"
        );

        let evidence_type = if dispute.seller_id == env::predecessor_account_id() {
            EvidenceType::Seller
        } else if dispute.buyer_id == env::predecessor_account_id() {
            EvidenceType::Buyer
        } else {
            env::panic_str("ERR_NOT_PARTICIPANT");
        };

        let evidence = Evidence {
            evidence_type,
            dispute_id,
            description,
            link,
            created_at: env::block_timestamp(),
        };

        let mut evidence_list = self
            .evidence_by_dispute_id
            .get(&dispute_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKeys::Evidence {
                        dispute_id_hash: dispute_id.try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });

        evidence_list.insert(&evidence);

        self.evidence_by_dispute_id
            .insert(&dispute_id, &evidence_list);
    }

    fn get_evidence(&self, dispute_id: u64) -> Vec<Evidence> {
        self.evidence_by_dispute_id
            .get(&dispute_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKeys::Evidence {
                        dispute_id_hash: dispute_id.try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            })
            .to_vec()
    }
}
