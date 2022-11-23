/**
 * DMS - Deer Market Standards
 * DMS297 - Extension of NEP-297 for store events
 */
use crate::*;
use serde_with::skip_serializing_none;

#[derive(Serialize, Debug)]
#[serde(tag = "standard")]
#[serde(rename_all = "snake_case")]
pub enum NearEvent {
    Dms297(Nep297Event),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nep297Event {
    pub version: &'static str,
    #[serde(flatten)]
    pub event_kind: Nep297EventKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum Nep297EventKind {
    // store
    StoreCreate(StoreCreateData),
    StoreUpdate(StoreUpdateData),
    StoreDelete(),
    // item
    ItemCreate(ItemCreateData),
    ItemUpdate(ItemUpdateData),
    ItemDelete(ItemDeleteData),
    // order
    ItemBuy(ItemBuyData),
    OrderShipped(OrderShippedData),
    OrderComplete(OrderCompleteData),
    OrderCancel(OrderCancelData),
    // dispute
    DisputeStart(DisputeStartData),
    DisputeResolve(DisputeResolveData),
    // review
    ReviewCreate(ReviewCreateData),
}

/**
 * event: store_create
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreCreateData {
    pub owner_id: AccountId,
    pub arbiter_id: AccountId,
    pub metadata: StoreMetadata,
}

impl StoreCreateData {
    pub fn new(owner_id: AccountId, arbiter_id: AccountId, metadata: StoreMetadata) -> Self {
        Self {
            owner_id: owner_id,
            arbiter_id: arbiter_id,
            metadata,
        }
    }
}

/**
 * event: store_update
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreUpdateData {
    pub owner_id: AccountId,
    pub arbiter_id: AccountId,
    pub metadata: StoreMetadata,
}

impl StoreUpdateData {
    pub fn new(owner_id: AccountId, arbiter_id: AccountId, metadata: StoreMetadata) -> Self {
        Self {
            owner_id: owner_id,
            arbiter_id: arbiter_id,
            metadata,
        }
    }
}

/**
 * event: item_create
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemCreateData {
    pub item_id: U64,
    pub price: U128,
    pub status: ItemStatus,
    pub metadata: ItemMetadata,
}

impl ItemCreateData {
    pub fn new(item_id: U64, price: U128, status: ItemStatus, metadata: ItemMetadata) -> Self {
        Self {
            item_id: item_id,
            price: price,
            status: status,
            metadata,
        }
    }
}

/**
 * event: item_update
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemUpdateData {
    pub item_id: U64,
    pub price: U128,
    pub status: ItemStatus,
    pub metadata: ItemMetadata,
}

impl ItemUpdateData {
    pub fn new(item_id: U64, price: U128, status: ItemStatus, metadata: ItemMetadata) -> Self {
        Self {
            item_id: item_id,
            price: price,
            status: status,
            metadata,
        }
    }
}

/**
 * event: item_delete
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemDeleteData {
    pub item_id: U64,
}

impl ItemDeleteData {
    pub fn new(item_id: U64) -> Self {
        Self { item_id: item_id }
    }
}

/**
 * event: item_buy
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemBuyData {
    pub item_id: U64,
    pub buyer_id: AccountId,
    pub order_id: U64,
    pub price: U128,
}

impl ItemBuyData {
    pub fn new(item_id: U64, buyer_id: AccountId, price: U128, order_id: U64) -> Self {
        Self {
            item_id: item_id,
            buyer_id: buyer_id,
            order_id: order_id,
            price: price,
        }
    }
}

/**
* event: order_shipped
*/

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderShippedData {
    pub order_id: U64,
}

impl OrderShippedData {
    pub fn new(order_id: U64) -> Self {
        Self { order_id: order_id }
    }
}

/**
 * event: order_complete
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderCompleteData {
    pub order_id: U64,
}

impl OrderCompleteData {
    pub fn new(order_id: U64) -> Self {
        Self { order_id: order_id }
    }
}

/**
 * event: order_cancel
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderCancelData {
    pub order_id: U64,
}

impl OrderCancelData {
    pub fn new(order_id: U64) -> Self {
        Self { order_id: order_id }
    }
}

/**
 * event: dispute_start
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct DisputeStartData {
    pub order_id: U64,
}

impl DisputeStartData {
    pub fn new(order_id: U64) -> Self {
        Self { order_id: order_id }
    }
}

/**
 * event: dispute_resolve
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct DisputeResolveData {
    pub order_id: U64,
    pub resolution: DisputeResolution,
}

impl DisputeResolveData {
    pub fn new(order_id: U64, resolution: DisputeResolution) -> Self {
        Self {
            order_id: order_id,
            resolution: resolution,
        }
    }
}

/**
 * event: review_create
 */

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct ReviewCreateData {
    pub item_id: U64,
    pub review_id: U64,
    pub reviewer_id: AccountId,
    pub rating: u8,
    pub comment: String,
}

impl ReviewCreateData {
    pub fn new(
        item_id: U64,
        review_id: U64,
        reviewer_id: AccountId,
        rating: u8,
        comment: String,
    ) -> Self {
        Self {
            item_id: item_id,
            review_id: review_id,
            reviewer_id: reviewer_id,
            rating: rating,
            comment,
        }
    }
}

impl NearEvent {
    pub fn new_event(event_kind: Nep297EventKind) -> Self {
        NearEvent::Dms297(Nep297Event {
            version: "0.0.1",
            event_kind,
        })
    }

    /**
     * Store events
     */
    pub fn store_create(data: StoreCreateData) -> Self {
        NearEvent::new_event(Nep297EventKind::StoreCreate(data))
    }

    pub fn store_update(data: StoreUpdateData) -> Self {
        NearEvent::new_event(Nep297EventKind::StoreUpdate(data))
    }

    pub fn store_delete() -> Self {
        NearEvent::new_event(Nep297EventKind::StoreDelete())
    }

    /**
     * Item events
     */
    pub fn item_create(data: ItemCreateData) -> Self {
        NearEvent::new_event(Nep297EventKind::ItemCreate(data))
    }

    pub fn item_update(data: ItemUpdateData) -> Self {
        NearEvent::new_event(Nep297EventKind::ItemUpdate(data))
    }

    pub fn item_delete(data: ItemDeleteData) -> Self {
        NearEvent::new_event(Nep297EventKind::ItemDelete(data))
    }

    /**
     * Order events
     */
    pub fn item_buy(data: ItemBuyData) -> Self {
        NearEvent::new_event(Nep297EventKind::ItemBuy(data))
    }

    pub fn order_shipped(data: OrderShippedData) -> Self {
        NearEvent::new_event(Nep297EventKind::OrderShipped(data))
    }

    pub fn order_complete(data: OrderCompleteData) -> Self {
        NearEvent::new_event(Nep297EventKind::OrderComplete(data))
    }

    pub fn order_cancel(data: OrderCancelData) -> Self {
        NearEvent::new_event(Nep297EventKind::OrderCancel(data))
    }

    /**
     * Dispute events
     */
    pub fn dispute_start(data: DisputeStartData) -> Self {
        NearEvent::new_event(Nep297EventKind::DisputeStart(data))
    }

    pub fn dispute_resolve(data: DisputeResolveData) -> Self {
        NearEvent::new_event(Nep297EventKind::DisputeResolve(data))
    }

    /**
     * Review events
     */
    pub fn review_create(data: ReviewCreateData) -> Self {
        NearEvent::new_event(Nep297EventKind::ReviewCreate(data))
    }

    /**
     * Helper functions
     */
    pub(crate) fn to_json_string(&self) -> String {
        near_sdk::serde_json::to_string(self).unwrap()
    }

    pub fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }

    /// Logs the event to the host. This is required to ensure that the event is triggered
    /// and to consume the event.
    pub fn emit(self) {
        near_sdk::env::log_str(&self.to_json_event_string());
    }
}
