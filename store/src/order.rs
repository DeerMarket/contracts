/**
 *  Orders
 *
 * Methods:
 *
 * - get_order
 *
 *
 * - item_buy
 * - order_complete
 * - order_cancel
 *
 *
 */
use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderStatus {
    Pending,
    Completed,
    Cancelled,
    Disputed,
    Resolved,
}

// Order
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    pub buyer_id: AccountId,
    pub item_id: u64,
    pub amount: Balance,
    pub status: OrderStatus,
    pub starts: u64,
    pub ends: Option<u64>,
}

pub trait OrderProvider {
    fn get_order(&self, order_id: U64) -> Option<Order>;
}

#[near_bindgen]
impl OrderProvider for Contract {
    fn get_order(&self, order_id: U64) -> Option<Order> {
        self.orders_by_id.get(&order_id.into())
    }
}

// Order Actions

pub trait OrderActions {
    fn item_buy(&mut self, item_id: U64) -> U64;
    fn order_complete(&mut self, order_id: U64) -> Promise;
    fn order_cancel(&mut self, order_id: U64) -> Promise;
}

#[near_bindgen]
impl OrderActions for Contract {
    #[payable]
    fn item_buy(&mut self, item_id: U64) -> U64 {
        //check if item exists
        require!(
            self.items_by_id.contains_key(&item_id.into()),
            "Item does not exist"
        );

        //check if item is available
        let item = self.items_by_id.get(&item_id.into()).unwrap();
        require!(item.status == ItemStatus::Active, "Item is not available");

        //check if enough attached deposit
        require!(
            env::attached_deposit() >= item.price,
            "Not enough deposit to buy this item"
        );

        //create the order
        let order_id = self.orders_by_id.len().into();
        let order = Order {
            buyer_id: env::predecessor_account_id(),
            item_id: item_id.into(),
            amount: item.price,
            status: OrderStatus::Pending,
            starts: env::block_timestamp_ms(),
            ends: None,
        };

        //save the order
        self.orders_by_id.insert(&order_id, &order);

        //save the order id for the buyer
        let mut buyer_order_ids = self
            .orders_by_account_id
            .get(&env::predecessor_account_id())
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::OrdersByAccountIdInner {
                        account_id_hash: env::predecessor_account_id().try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        buyer_order_ids.insert(&order_id);
        self.orders_by_account_id
            .insert(&env::predecessor_account_id(), &buyer_order_ids);

        //save the order id for the item
        let mut item_order_ids = self
            .orders_by_item_id
            .get(&item_id.into())
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::OrdersByItemIdInner {
                        item_id_hash: item_id.try_to_vec().unwrap(),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        item_order_ids.insert(&order_id);
        self.orders_by_item_id
            .insert(&item_id.into(), &item_order_ids);

        // Emit NearEvent
        NearEvent::item_buy(ItemBuyData::new(
            item_id,
            env::predecessor_account_id(),
            U128(item.price),
            U64(order_id),
        ))
        .emit();

        //return the order id
        U64(order_id)
    }

    fn order_complete(&mut self, order_id: U64) -> Promise {
        //get the order
        let mut order = self.orders_by_id.get(&order_id.into()).unwrap();

        //check if order is pending
        require!(order.status == OrderStatus::Pending, "Order is not pending");

        //get the buyer id
        let buyer_id = env::predecessor_account_id();

        //check if buyer is the buyer
        require!(
            order.buyer_id == buyer_id,
            "Only the buyer can complete the order"
        );

        //update the order status
        order.status = OrderStatus::Completed;

        //save the order
        self.orders_by_id.insert(&order_id.into(), &order);

        // Emit NearEvent
        NearEvent::order_complete(OrderCompleteData::new(order_id)).emit();

        //transfer the amount to the seller
        Promise::new(self.owner_id.to_owned()).transfer(order.amount)
    }

    fn order_cancel(&mut self, order_id: U64) -> Promise {
        //get the order
        let mut order = self.orders_by_id.get(&order_id.into()).unwrap();

        //check if order is pending
        require!(order.status == OrderStatus::Pending, "Order is not pending");

        //get the caller id
        let caller_id = env::predecessor_account_id();

        //check if caller is the owner
        require!(
            caller_id == self.owner_id,
            "Only the owner can cancel the order"
        );

        //update the order status
        order.status = OrderStatus::Cancelled;

        //save the order
        self.orders_by_id.insert(&order_id.into(), &order);

        //get the buyer id
        let buyer_id = order.buyer_id;

        // Emit NearEvent
        NearEvent::order_cancel(OrderCancelData::new(order_id)).emit();

        //refund the amount to the buyer
        Promise::new(buyer_id).transfer(order.amount)
    }
}
