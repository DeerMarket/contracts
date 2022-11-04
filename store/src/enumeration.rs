/**
 *
 *  Enumeration
 *
 * Methods:
 *
 * - get_items
 *
 * - get_orders
 * - get_orders_for_item
 * - get_orders_for_buyer
 *
 * - get_reviews
 * - get_reviews_for_item
 * - get_reviews_for_buyer
 *
 */
use crate::*;

pub trait Enumeration {
    fn get_items(&self, from_index: Option<U64>, limit: Option<U64>) -> Vec<JsonItem>;
    fn get_orders(&self, from_index: Option<U64>, limit: Option<U64>) -> Vec<Order>;
    fn get_orders_for_item(
        &self,
        item_id: U64,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<Order>;
    fn get_orders_for_buyer(
        &self,
        buyer_id: AccountId,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<Order>;
    fn get_reviews(&self, from_index: Option<U64>, limit: Option<U64>) -> Vec<Review>;
    fn get_reviews_for_item(
        &self,
        item_id: U64,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<Review>;
    fn get_reviews_for_buyer(
        &self,
        buyer_id: AccountId,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<Review>;
}

#[near_bindgen]
impl Enumeration for Contract {
    fn get_items(&self, from_index: Option<U64>, limit: Option<U64>) -> Vec<JsonItem> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(U64(0)).0;

        //iterate through each item using an iterator
        self.items_metadata_by_id
            .keys()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(U64(50)).0 as usize)
            //we'll map the item IDs which are strings into Json Tokens
            .map(|item_id| self.get_item(U64(item_id)).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    fn get_orders(&self, from_index: Option<U64>, limit: Option<U64>) -> Vec<Order> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(U64(0)).0;

        //iterate through each item using an iterator
        self.orders_by_id
            .keys()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(U64(50)).0 as usize)
            //we'll map the item IDs which are strings into Json
            .map(|order_id| self.get_order(U64(order_id)).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    fn get_orders_for_item(
        &self,
        item_id: U64,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<Order> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(U64(0)).0;

        //iterate through each item using an iterator
        self.orders_by_item_id
            .get(&item_id.into())
            .unwrap()
            .iter()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(U64(50)).0 as usize)
            //we'll map the item IDs which are strings into Json
            .map(|order_id| self.get_order(U64(order_id)).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    fn get_orders_for_buyer(
        &self,
        buyer_id: AccountId,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<Order> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(U64(0)).0;

        //iterate through each item using an iterator
        self.orders_by_account_id
            .get(&buyer_id)
            .unwrap()
            .iter()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(U64(50)).0 as usize)
            //we'll map the item IDs which are strings into Json
            .map(|order_id| self.get_order(U64(order_id)).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    fn get_reviews(&self, from_index: Option<U64>, limit: Option<U64>) -> Vec<Review> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(U64(0)).0;

        //iterate through each item using an iterator
        self.reviews_by_id
            .keys()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(U64(50)).0 as usize)
            //we'll map the item IDs which are strings into Json
            .map(|review_id| self.get_review(U64(review_id)).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    fn get_reviews_for_item(
        &self,
        item_id: U64,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<Review> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(U64(0)).0;

        //iterate through each item using an iterator
        self.reviews_by_item_id
            .get(&item_id.into())
            .unwrap()
            .iter()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(U64(50)).0 as usize)
            //we'll map the item IDs which are strings into Json
            .map(|review_id| self.get_review(U64(review_id)).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    fn get_reviews_for_buyer(
        &self,
        buyer_id: AccountId,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<Review> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(U64(0)).0;

        //iterate through each item using an iterator
        self.reviews_by_account_id
            .get(&buyer_id)
            .unwrap()
            .iter()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(U64(50)).0 as usize)
            //we'll map the item IDs which as strings into Json
            .map(|review_id| self.get_review(U64(review_id)).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }
}
