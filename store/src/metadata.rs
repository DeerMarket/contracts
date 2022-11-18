/**
 *  Metadata
 *
 * Methods:
 * - update_store_metadata
 *
 * - store_metadata
 * - get_store_owner
 * - get_store_arbiter
 *
 *
 */
use crate::*;

// Store metadata

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StoreMetadata {
    // important values
    pub name: String, // required, ex. "My Store"
    pub category: u8, // required, ex. 1 - Note: categories are defined by this key in the frontend or the indexers

    // identity values
    pub description: Option<String>, // optional, ex. "My Store sells awesome stuff"
    pub logo: Option<String>,        // optional, ex. "https://example.com/image.png"
    pub cover: Option<String>,       // optional, ex. "https://example.com/image.png"

    // contact values, at least one way of contact is required
    pub website: Option<String>, // optional
    pub email: Option<String>,   // optional
    pub phone: Option<String>,   // optional

    // terms values
    pub terms: Option<String>, // optional, should be a short version of the store terms

    // other values
    pub tags: Option<Vec<String>>, // optional, ex. ["store", "awesome"]
    pub created_at: Option<String>, // optional, timestamp of creation
    pub updated_at: Option<String>, // optional, timestamp of last update to metadata
}

pub trait StoreMetadataProvider {
    //view call for returning the contract metadata
    fn store_metadata(&self) -> StoreMetadata;
    // get store owner
    fn get_store_owner(&self) -> AccountId;
    // get store arbiter
    fn get_store_arbiter(&self) -> AccountId;
}

#[near_bindgen]
impl StoreMetadataProvider for Contract {
    fn store_metadata(&self) -> StoreMetadata {
        self.metadata.get().unwrap()
    }
    fn get_store_owner(&self) -> AccountId {
        self.owner_id.clone()
    }
    fn get_store_arbiter(&self) -> AccountId {
        self.arbiter_id.clone()
    }
}

pub trait StoreMetadataManager {
    //update the contract metadata
    fn update_store_metadata(&mut self, metadata: StoreMetadata);
}

#[near_bindgen]
impl StoreMetadataManager for Contract {
    fn update_store_metadata(&mut self, metadata: StoreMetadata) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only owner can update store metadata"
        );

        let orders = &self.orders_by_id;
        for order in orders.values() {
            assert!(
                order.status == OrderStatus::Pending
                    || order.status == OrderStatus::Shipped
                    || order.status == OrderStatus::Disputed,
                "Can't update store metadata while there are active orders or disputes"
            );
        }

        self.metadata.set(&StoreMetadata {
            created_at: self.metadata.get().unwrap().created_at,
            updated_at: Some(env::block_timestamp_ms().to_string()),
            ..metadata
        });

        // Emit NearEvent
        NearEvent::store_update(StoreUpdateData::new(
            self.owner_id.clone(),
            self.arbiter_id.clone(),
            self.metadata.get().unwrap(),
        ))
        .emit();
    }
}

// Item Metadata

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ItemMetadata {
    pub title: String,               // required, ex. "My Item"
    pub description: Option<String>, // optional, ex. "My Item is awesome"
    pub images: Option<Vec<String>>, // optional, ex. ["https://example.com/image.png"]

    pub tags: Option<Vec<String>>, // optional, ex. ["music", "guitar"]
}
