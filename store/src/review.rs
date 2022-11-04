/**
 *  Review
 *
 * Methods:
 *
 * - get_review
 *
 * - item_review
 *  
 *
 */
use crate::*;

// Review
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Review {
    pub reviewer_id: AccountId,
    pub rating: u8,
    pub comment: String,
}

pub trait ReviewProvider {
    fn get_review(&self, review_id: U64) -> Option<Review>;
}

#[near_bindgen]
impl ReviewProvider for Contract {
    fn get_review(&self, review_id: U64) -> Option<Review> {
        self.reviews_by_id.get(&review_id.into())
    }
}

pub trait ReviewManager {
    fn item_review(&mut self, item_id: U64, rating: u8, comment: Option<String>) -> U64;
}

#[near_bindgen]
impl ReviewManager for Contract {
    fn item_review(&mut self, item_id: U64, rating: u8, comment: Option<String>) -> U64 {
        // Check if item exists
        require!(
            self.items_by_id.contains_key(&item_id.into()),
            "Item does not exist"
        );

        // check if user has purchased item
        self.orders_by_account_id
            .get(&env::predecessor_account_id())
            .unwrap()
            .iter()
            .for_each(|order_id| {
                let order = self.orders_by_id.get(&order_id).unwrap();
                require!(
                    order.item_id == u64::from(item_id),
                    "You have not purchased this item"
                );
                require!(
                    order.status == OrderStatus::Completed,
                    "Order is not completed"
                );
            });

        require!(rating <= 5, "Rating must be between 0 and 5");

        let review_id = self.reviews_by_id.len();
        let review = Review {
            reviewer_id: env::predecessor_account_id(),
            rating,
            comment: comment.clone().unwrap_or("".to_string()),
        };
        self.reviews_by_id.insert(&review_id, &review);

        let mut users_reviews_ids = self
            .reviews_by_account_id
            .get(&env::predecessor_account_id())
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::ReviewsByAccountIdInner {
                        account_id_hash: env::predecessor_account_id().try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        users_reviews_ids.insert(&review_id);
        self.reviews_by_account_id
            .insert(&env::predecessor_account_id(), &users_reviews_ids);

        let mut item_reviews_ids =
            self.reviews_by_item_id
                .get(&item_id.into())
                .unwrap_or_else(|| {
                    UnorderedSet::new(
                        StorageKey::ReviewsByItemIdInner {
                            item_id_hash: item_id.try_to_vec().unwrap(),
                        }
                        .try_to_vec()
                        .unwrap(),
                    )
                });
        item_reviews_ids.insert(&review_id);
        self.reviews_by_item_id
            .insert(&item_id.into(), &item_reviews_ids);

        // emit NearEvent
        NearEvent::review_create(ReviewCreateData::new(
            item_id,
            U64(review_id),
            env::predecessor_account_id(),
            rating,
            comment.unwrap_or("".to_string()),
        ))
        .emit();

        U64(review_id)
    }
}
