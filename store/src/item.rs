/**
 *  Item
 *
 * Methods:
 *
 * - get_item
 *
 * - item_create
 * - item_update
 * - item_delete
 *
 *
 */
use crate::*;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Item {
    pub price: Balance,
    pub status: ItemStatus,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ItemStatus {
    Active,
    Inactive,
}

// The Json Item is what will be returned from view calls.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonItem {
    pub id: U64,
    pub price: U128,
    pub status: ItemStatus,
    pub metadata: ItemMetadata,
}

pub trait ItemProvider {
    fn get_item(&self, item_id: U64) -> Option<JsonItem>;
}

#[near_bindgen]
impl ItemProvider for Contract {
    fn get_item(&self, item_id: U64) -> Option<JsonItem> {
        if let Some(item) = self.items_by_id.get(&item_id.into()) {
            let metadata = self.items_metadata_by_id.get(&item_id.into()).unwrap();
            Some(JsonItem {
                id: item_id,
                price: item.price.into(),
                status: item.status,
                metadata,
            })
        } else {
            None
        }
    }
}

pub trait ItemManager {
    fn item_create(&mut self, price: U128, metadata: ItemMetadata) -> U64;
    fn item_update(&mut self, item_id: U64, price: U128, metadata: ItemMetadata);
    fn item_delete(&mut self, item_id: U64);
}

#[near_bindgen]
impl ItemManager for Contract {
    fn item_create(&mut self, price: U128, metadata: ItemMetadata) -> U64 {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only owner can create a item"
        );
        let item_id = self.items_metadata_by_id.len();
        let item = Item {
            price: price.into(),
            status: ItemStatus::Active,
        };
        self.items_by_id.insert(&item_id, &item);
        self.items_metadata_by_id.insert(&item_id, &metadata);

        // Emit a NearEvent
        NearEvent::item_create(ItemCreateData::new(
            U64(item_id),
            price,
            item.status,
            metadata,
        ))
        .emit();

        U64(item_id)
    }

    fn item_update(&mut self, item_id: U64, price: U128, metadata: ItemMetadata) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only owner can update a item"
        );

        let orders = &self.orders_by_item_id.get(&item_id.into());
        if orders.is_some() {
            orders.as_ref().unwrap().iter().for_each(|order_id| {
                let order = self.orders_by_id.get(&order_id).unwrap();
                assert!(
                    order.status == OrderStatus::Pending || order.status == OrderStatus::Disputed,
                    "Can't update item while there are active orders or disputes for this item"
                );
            });
        }

        let mut item = self.items_by_id.get(&item_id.into()).unwrap();
        item.price = price.into();
        self.items_by_id.insert(&item_id.into(), &item);
        self.items_metadata_by_id.insert(&item_id.into(), &metadata);

        // Emit a NearEvent
        NearEvent::item_update(ItemUpdateData::new(item_id, price, item.status, metadata)).emit();
    }

    fn item_delete(&mut self, item_id: U64) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only owner can delete a item"
        );

        let orders = &self.orders_by_item_id.get(&item_id.into());
        if orders.is_some() {
            orders.as_ref().unwrap().iter().for_each(|order_id| {
                let order = self.orders_by_id.get(&order_id).unwrap();
                assert!(
                    order.status == OrderStatus::Pending || order.status == OrderStatus::Disputed,
                    "Can't update item while there are active orders or disputes for this item"
                );
            });
        }

        self.items_by_id.remove(&item_id.into());
        self.items_metadata_by_id.remove(&item_id.into());

        // Emit a NearEvent
        NearEvent::item_delete(ItemDeleteData::new(item_id)).emit();
    }
}
