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
    BuyerWon,  // Buyer wins
    SellerWon, // Seller wins
    Draw,      // Split the funds
}

pub trait DisputeManager {
    fn start_dispute(&mut self, order_id: U64, description: String) -> Promise;
    fn start_dispute_callback(
        &mut self,
        order_id: U64,
        new_amount: Balance,
        call_result: Result<String, PromiseError>,
    );
    fn dispute_resolve(&mut self, order_id: U64, resolution: DisputeResolution) -> Promise;
}

#[near_bindgen]
impl DisputeManager for Contract {
    fn start_dispute(&mut self, order_id: U64, description: String) -> Promise {
        //only buyer or owner can start dispute

        let order = self.orders_by_id.get(&order_id.into()).unwrap();
        let owner_id = self.owner_id.clone();

        require!(
            env::predecessor_account_id() == order.buyer_id
                || env::predecessor_account_id() == owner_id,
            "Only buyer or owner can start dispute"
        );

        //only shipped orders can be disputed
        require!(
            order.status == OrderStatus::Shipped,
            "Only shipped orders can be disputed"
        );

        // 20% of the funds goes to the arbitrator contract
        let arbitrator_amount = order.amount * 20 / 100;
        let new_amount = order.amount - arbitrator_amount;

        let dispute_args: Vec<u8> = near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({
            "store_id": env::current_account_id(),
            "item_id": order.item_id.to_string(),
            "order_id": order_id,
            "buyer_id": order.buyer_id,
            "seller_id": self.owner_id,
            "description": description,
        }))
        .unwrap();

        // cross contract call to arbitration contract (dispute contract)
        let promise = Promise::new(self.arbiter_id.clone()).function_call(
            "create_dispute".to_string(),
            dispute_args,
            arbitrator_amount,
            Gas::ONE_TERA * 10,
        );

        // callback to store contract
        let callback_args: Vec<u8> = near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({
            "order_id": order_id,
            "new_amount": new_amount,
        }))
        .expect("Failed to serialize callback args");

        return promise.then(Promise::new(env::current_account_id()).function_call(
            "start_dispute_callback".to_string(),
            callback_args,
            0,
            Gas::ONE_TERA * 10,
        ));
    }

    #[private]
    fn start_dispute_callback(
        &mut self,
        order_id: U64,
        new_amount: Balance,
        #[callback_result] call_result: Result<String, PromiseError>,
    ) {
        // Check if the promise succeeded
        if call_result.is_err() || !near_sdk::is_promise_success() {
            env::panic_str("Dispute creation failed..");
        }

        let dispute_id: u64 = near_sdk::serde_json::from_str(&call_result.unwrap()).unwrap();

        let mut order = self.orders_by_id.get(&order_id.into()).unwrap();
        order.status = OrderStatus::Disputed;
        order.amount = new_amount;
        order.dispute_id = Some(dispute_id);

        self.orders_by_id.insert(&order_id.into(), &order);        

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
        let order = self.orders_by_id.get(&order_id.into()).unwrap_or_else(|| {
            env::panic_str(&format!(
                "Order does not exist"
            ))
        });

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
            DisputeResolution::BuyerWon => Promise::new(buyer_id).transfer(order.amount),
            DisputeResolution::SellerWon => Promise::new(seller_id).transfer(order.amount),
            DisputeResolution::Draw => Promise::new(buyer_id)
                .transfer(order.amount / 2)
                .and(Promise::new(seller_id).transfer(order.amount / 2)),
        }
    }
}
