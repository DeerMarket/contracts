use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, Balance, Gas, PanicOnDefault, Promise};

mod dispute;
mod enumeration;
mod event;
mod item;
mod metadata;
mod order;
mod review;

#[allow(unused_imports)]
use crate::dispute::*;
#[allow(unused_imports)]
use crate::enumeration::*;
use crate::event::*;
use crate::item::*;
use crate::metadata::*;
use crate::order::*;
use crate::review::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Store owner
    pub owner_id: AccountId,

    /// Store middleman
    pub arbiter_id: AccountId,

    /// Store metadata
    pub metadata: LazyOption<StoreMetadata>,

    /// Items
    pub items_by_id: LookupMap<u64, Item>,
    pub items_metadata_by_id: UnorderedMap<u64, ItemMetadata>,

    /// Orders
    pub orders_by_id: UnorderedMap<u64, Order>,
    pub orders_by_account_id: LookupMap<AccountId, UnorderedSet<u64>>,
    pub orders_by_item_id: LookupMap<u64, UnorderedSet<u64>>,

    /// Reviews
    pub reviews_by_id: UnorderedMap<u64, Review>,
    pub reviews_by_account_id: LookupMap<AccountId, UnorderedSet<u64>>,
    pub reviews_by_item_id: LookupMap<u64, UnorderedSet<u64>>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    StoreMetadata,
    ItemsById,
    ItemsMetadataById,
    OrdersById,
    OrdersByAccountId,
    OrdersByAccountIdInner { account_id_hash: Vec<u8> },
    OrdersByItemId,
    OrdersByItemIdInner { item_id_hash: Vec<u8> },
    ReviewsById,
    ReviewsByAccountId,
    ReviewsByAccountIdInner { account_id_hash: Vec<u8> },
    ReviewsByItemId,
    ReviewsByItemIdInner { item_id_hash: Vec<u8> },
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, metadata: StoreMetadata) -> Self {
        require!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Owner's account ID is invalid"
        );

        // overriding arbiter_id currently
        let arbiter_id: AccountId = "ddd5.testnet".parse().unwrap();

        let metadata = StoreMetadata {
            created_at: Some(env::block_timestamp_ms().to_string()),
            updated_at: Some(env::block_timestamp_ms().to_string()),
            ..metadata.clone()
        };

        let this = Self {
            owner_id: owner_id.clone(),
            arbiter_id: arbiter_id.clone(),
            metadata: LazyOption::new(
                StorageKey::StoreMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            items_by_id: LookupMap::new(StorageKey::ItemsById.try_to_vec().unwrap()),
            items_metadata_by_id: UnorderedMap::new(
                StorageKey::ItemsMetadataById.try_to_vec().unwrap(),
            ),
            orders_by_id: UnorderedMap::new(StorageKey::OrdersById.try_to_vec().unwrap()),
            orders_by_account_id: LookupMap::new(
                StorageKey::OrdersByAccountId.try_to_vec().unwrap(),
            ),
            orders_by_item_id: LookupMap::new(StorageKey::OrdersByItemId.try_to_vec().unwrap()),
            reviews_by_id: UnorderedMap::new(StorageKey::ReviewsById.try_to_vec().unwrap()),
            reviews_by_account_id: LookupMap::new(
                StorageKey::ReviewsByAccountId.try_to_vec().unwrap(),
            ),
            reviews_by_item_id: LookupMap::new(StorageKey::ReviewsByItemId.try_to_vec().unwrap()),
        };

        // Emit a NearEvent
        NearEvent::store_create(StoreCreateData::new(owner_id, arbiter_id, metadata)).emit();

        this
    }

    // Delete the contract and refund remaining balance to the owner.
    pub fn delete_self(&mut self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Can only be called by the owner"
        );

        let orders = &self.orders_by_id;
        for order in orders.values() {
            require!(
                order.status == OrderStatus::Pending
                    || order.status == OrderStatus::Shipped
                    || order.status == OrderStatus::Disputed,
                "Can't delete store with orders in progress"
            );
        }

        env::storage_remove(&StorageKey::StoreMetadata.try_to_vec().unwrap());
        env::storage_remove(&StorageKey::ItemsById.try_to_vec().unwrap());
        env::storage_remove(&StorageKey::ItemsMetadataById.try_to_vec().unwrap());
        env::storage_remove(&StorageKey::OrdersById.try_to_vec().unwrap());
        env::storage_remove(&StorageKey::OrdersByAccountId.try_to_vec().unwrap());
        env::storage_remove(&StorageKey::OrdersByItemId.try_to_vec().unwrap());
        env::storage_remove(&StorageKey::ReviewsById.try_to_vec().unwrap());
        env::storage_remove(&StorageKey::ReviewsByAccountId.try_to_vec().unwrap());
        env::storage_remove(&StorageKey::ReviewsByItemId.try_to_vec().unwrap());

        // Emit a NearEvent
        NearEvent::store_delete().emit();

        let factory_id: AccountId = env::current_account_id()
            .as_str()
            .split('.')
            .collect::<Vec<&str>>()[1..]
            .join(".")
            .parse()
            .unwrap();

        // call factory to delete the store
        let cb_args: Vec<u8> = near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({
            "store_id": env::current_account_id(),
            "owner_id": self.owner_id,
        }))
        .unwrap();

        Promise::new(env::current_account_id()).delete_account(self.owner_id.clone());

        Promise::new(factory_id).function_call("remove".to_string(), cb_args, 0, Gas::ONE_TERA * 5);
    }
}

#[cfg(test)]
mod tests;
