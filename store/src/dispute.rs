/**
 *  Dispute
 *
 * Methods:
 *
 * - start_dispute
 * - dispute_resolve
 *
 */
use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum DisputeResolution {
    Buyer,   // Buyer wins
    Seller,  // Seller wins
    Neutral, // Split the funds
}

pub trait DisputeManager {
    fn start_dispute(&mut self, order_id: U64);
    fn dispute_resolve(&mut self, order_id: U64, resolution: DisputeResolution) -> Promise;
}

#[near_bindgen]
impl DisputeManager for Contract {
    fn start_dispute(&mut self, order_id: U64) {
        //only buyer or owner can start dispute

        let order = self.orders_by_id.get(&order_id.into()).unwrap();
        let owner_id = self.owner_id.clone();

        require!(
            env::predecessor_account_id() == order.buyer_id
                || env::predecessor_account_id() == owner_id,
            "Only buyer or owner can start dispute"
        );

        //only pending orders can be disputed
        require!(
            order.status == OrderStatus::Pending,
            "Only pending orders can be disputed"
        );

        //set order status to disputed
        self.orders_by_id.insert(
            &order_id.into(),
            &Order {
                status: OrderStatus::Disputed,
                ..order
            },
        );

        // emit NearEvent
        NearEvent::dispute_start(DisputeStartData::new(order_id)).emit();
    }

    fn dispute_resolve(&mut self, order_id: U64, resolution: DisputeResolution) -> Promise {
        let arbiter_id = self.arbiter_id.clone();

        //only arbiter can resolve dispute
        require!(
            env::predecessor_account_id() == arbiter_id,
            "Only arbiter can resolve dispute"
        );

        //get order
        let order = self.orders_by_id.get(&order_id.into()).unwrap();

        //only disputed orders can be resolved
        require!(
            order.status == OrderStatus::Disputed,
            "Only disputed orders can be resolved"
        );

        let seller_id = self.owner_id.clone();
        let buyer_id = order.buyer_id.clone();

        //set order status to resolved
        self.orders_by_id.insert(
            &order_id.into(),
            &Order {
                status: OrderStatus::Resolved,
                ..order
            },
        );

        // emit NearEvent
        NearEvent::dispute_resolve(DisputeResolveData::new(order_id, resolution.clone())).emit();

        //transfer funds
        match resolution {
            DisputeResolution::Buyer => Promise::new(buyer_id).transfer(order.amount),
            DisputeResolution::Seller => Promise::new(seller_id).transfer(order.amount),
            DisputeResolution::Neutral => Promise::new(buyer_id)
                .transfer(order.amount / 2)
                .and(Promise::new(seller_id).transfer(order.amount / 2)),
        }
    }
}
