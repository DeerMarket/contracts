#[cfg(test)]
use crate::*;
use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::testing_env;
use near_sdk::AccountId;

const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000;

fn get_context(predecessor: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder.predecessor_account_id(predecessor);
    builder
}

fn sample_store_metadata() -> StoreMetadata {
    StoreMetadata {
        name: "Test Store".to_string(),
        category: 1,
        description: Some("Test Store sells awesome stuff".to_string()),
        logo: Some("https://example.com/image.png".to_string()),
        cover: Some("https://example.com/image.png".to_string()),
        website: Some("https://example.com".to_string()),
        email: Some("a@example.com".to_string()),
        phone: Some("1234567890".to_string()),
        terms: Some("Short version of the store terms".to_string()),
        tags: Some(vec!["store".to_string(), "awesome".to_string()]),
        created_at: Some(1234567890.to_string()),
        updated_at: Some(1234567890.to_string()),
    }
}

fn sample_item_metadata() -> ItemMetadata {
    ItemMetadata {
        title: "My Item".to_string(),
        description: Some("My Item is awesome".to_string()),
        images: Some(vec![
            "https://example.com/image.png".to_string(),
            "https://example.com/image2.png".to_string(),
        ]),
        tags: Some(vec!["awesome".to_string(), "cool".to_string()]),
    }
}

#[test]
fn test_store_metadata() {
    let context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0), accounts(1), sample_store_metadata());

    assert_eq!(contract.store_metadata().name, sample_store_metadata().name);
    let new_metadata = StoreMetadata {
        name: "New Test Store".to_string(),
        ..sample_store_metadata()
    };
    contract.update_store_metadata(new_metadata.clone());
    assert_eq!(contract.store_metadata().name, new_metadata.name,);
}

#[test]
fn test_item_management() {
    let context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0), accounts(1), sample_store_metadata());
    let item_id = contract.item_create(U128(ONE_NEAR), sample_item_metadata());
    assert_eq!(
        contract
            .get_item(item_id)
            .unwrap()
            .metadata
            .try_to_vec()
            .unwrap(),
        sample_item_metadata().try_to_vec().unwrap()
    );
    assert_eq!(
        u128::from(contract.get_item(item_id).unwrap().price),
        ONE_NEAR
    );
    let new_metadata = ItemMetadata {
        title: "My New Item".to_string(),
        description: Some("My New Item is awesome".to_string()),
        images: Some(vec![
            "https://example.com/image.png".to_string(),
            "https://example.com/image2.png".to_string(),
        ]),
        tags: Some(vec!["awesome".to_string(), "cool".to_string()]),
    };
    contract.item_update(item_id, U128(ONE_NEAR * 2), new_metadata.clone());
    assert_eq!(
        contract
            .get_item(item_id)
            .unwrap()
            .metadata
            .try_to_vec()
            .unwrap(),
        new_metadata.try_to_vec().unwrap()
    );
    assert_eq!(
        u128::from(contract.get_item(item_id).unwrap().price),
        ONE_NEAR * 2
    );
    contract.item_delete(item_id);
    assert_eq!(contract.get_item(item_id).is_none(), true);
}

#[test]
fn test_order_management() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0), accounts(1), sample_store_metadata());
    let item_id = contract.item_create(U128(ONE_NEAR), sample_item_metadata());

    testing_env!(context
        .attached_deposit(ONE_NEAR)
        .predecessor_account_id(accounts(2))
        .build());
    let order_id = contract.item_buy(item_id);

    assert_eq!(contract.get_order(order_id).unwrap().buyer_id, accounts(2));
    assert_eq!(
        contract.get_order(order_id).unwrap().item_id,
        u64::from(item_id)
    );
    assert_eq!(contract.get_order(order_id).unwrap().amount, ONE_NEAR);
    assert_eq!(
        contract
            .get_order(order_id)
            .unwrap()
            .status
            .try_to_vec()
            .unwrap(),
        OrderStatus::Pending.try_to_vec().unwrap()
    );
    contract.order_complete(order_id);

    assert_eq!(
        contract
            .get_order(order_id)
            .unwrap()
            .status
            .try_to_vec()
            .unwrap(),
        OrderStatus::Completed.try_to_vec().unwrap()
    );

    let order_id = contract.item_buy(item_id);

    testing_env!(context.predecessor_account_id(accounts(0)).build());

    contract.order_cancel(order_id);
    assert_eq!(
        contract
            .get_order(order_id)
            .unwrap()
            .status
            .try_to_vec()
            .unwrap(),
        OrderStatus::Cancelled.try_to_vec().unwrap()
    );
}

#[test]
fn test_dispute_management() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0), accounts(1), sample_store_metadata());
    let item_id = contract.item_create(U128(ONE_NEAR), sample_item_metadata());

    testing_env!(context
        .attached_deposit(ONE_NEAR)
        .predecessor_account_id(accounts(2))
        .build());
    let order_id = contract.item_buy(item_id);

    testing_env!(context
        .predecessor_account_id(accounts(0))
        .attached_deposit(ONE_NEAR)
        .build());
    contract.start_dispute(order_id);

    assert_eq!(
        contract
            .get_order(order_id)
            .unwrap()
            .status
            .try_to_vec()
            .unwrap(),
        OrderStatus::Disputed.try_to_vec().unwrap()
    );

    testing_env!(context.predecessor_account_id(accounts(1)).build());
    contract.dispute_resolve(order_id, DisputeResolution::Buyer);

    assert_eq!(
        contract
            .get_order(order_id)
            .unwrap()
            .status
            .try_to_vec()
            .unwrap(),
        OrderStatus::Resolved.try_to_vec().unwrap()
    );
}

#[test]
fn test_review_management() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0), accounts(1), sample_store_metadata());
    let item_id = contract.item_create(U128(ONE_NEAR), sample_item_metadata());

    testing_env!(context
        .attached_deposit(ONE_NEAR)
        .predecessor_account_id(accounts(2))
        .build());
    let order_id = contract.item_buy(item_id);

    contract.order_complete(order_id);

    assert_eq!(
        contract
            .get_order(order_id)
            .unwrap()
            .status
            .try_to_vec()
            .unwrap(),
        OrderStatus::Completed.try_to_vec().unwrap()
    );

    let review_id = contract.item_review(item_id.clone(), 5, Some("Awesome".to_string()));

    let review = contract.get_review(review_id).unwrap();

    assert_eq!(review.reviewer_id, accounts(2));
    assert_eq!(review.rating, 5);
    assert_eq!(review.comment, "Awesome".to_string());
}

#[test]
fn test_enumeration() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0), accounts(1), sample_store_metadata());
    let item_id = contract.item_create(U128(ONE_NEAR), sample_item_metadata());

    testing_env!(context
        .attached_deposit(ONE_NEAR)
        .predecessor_account_id(accounts(2))
        .build());
    let order_id = contract.item_buy(item_id);
    contract.order_complete(order_id);
    contract.item_review(item_id.clone(), 5, Some("Awesome".to_string()));

    testing_env!(context
        .attached_deposit(ONE_NEAR)
        .predecessor_account_id(accounts(3))
        .build());

    let order_id = contract.item_buy(item_id);
    contract.order_complete(order_id);
    let review_id = contract.item_review(item_id.clone(), 5, Some("Awesome".to_string()));

    assert!(contract.get_review(review_id).is_some());

    testing_env!(context
        .attached_deposit(ONE_NEAR)
        .predecessor_account_id(accounts(4))
        .build());

    let order_id = contract.item_buy(item_id);
    contract.order_complete(order_id);
    contract.item_review(item_id.clone(), 5, Some("Awesome".to_string()));

    assert_eq!(contract.get_items(None, None).len(), 1);
    assert_eq!(contract.get_items(None, Some(U64(1))).len(), 1);
    assert_eq!(contract.get_items(Some(U64(1)), None).len(), 0);
    assert_eq!(contract.get_items(Some(U64(1)), Some(U64(1))).len(), 0);

    assert_eq!(contract.get_orders(None, None).len(), 3);
    assert_eq!(contract.get_orders(None, Some(U64(1))).len(), 1);
    assert_eq!(contract.get_orders(Some(U64(1)), None).len(), 2);
    assert_eq!(contract.get_orders(Some(U64(1)), Some(U64(1))).len(), 1);

    assert_eq!(contract.get_orders_for_item(item_id, None, None).len(), 3);
    assert_eq!(
        contract
            .get_orders_for_item(item_id, None, Some(U64(1)))
            .len(),
        1
    );
    assert_eq!(
        contract
            .get_orders_for_item(item_id, Some(U64(1)), None)
            .len(),
        2
    );
    assert_eq!(
        contract
            .get_orders_for_item(item_id, Some(U64(1)), Some(U64(1)))
            .len(),
        1
    );

    assert_eq!(
        contract.get_orders_for_buyer(accounts(2), None, None).len(),
        1
    );
    assert_eq!(
        contract
            .get_orders_for_buyer(accounts(2), None, Some(U64(1)))
            .len(),
        1
    );
    assert_eq!(
        contract
            .get_orders_for_buyer(accounts(2), Some(U64(1)), None)
            .len(),
        0
    );
    assert_eq!(
        contract
            .get_orders_for_buyer(accounts(2), Some(U64(1)), Some(U64(1)))
            .len(),
        0
    );

    assert_eq!(contract.get_reviews(None, None).len(), 3);
    assert_eq!(contract.get_reviews(None, Some(U64(1))).len(), 1);
    assert_eq!(contract.get_reviews(Some(U64(1)), None).len(), 2);
    assert_eq!(contract.get_reviews(Some(U64(1)), Some(U64(1))).len(), 1);

    assert_eq!(contract.get_reviews_for_item(item_id, None, None).len(), 3);
    assert_eq!(
        contract
            .get_reviews_for_item(item_id, None, Some(U64(1)))
            .len(),
        1
    );
    assert_eq!(
        contract
            .get_reviews_for_item(item_id, Some(U64(1)), None)
            .len(),
        2
    );
    assert_eq!(
        contract
            .get_reviews_for_item(item_id, Some(U64(1)), Some(U64(1)))
            .len(),
        1
    );

    assert_eq!(
        contract
            .get_reviews_for_buyer(accounts(2), None, None)
            .len(),
        1
    );
    assert_eq!(
        contract
            .get_reviews_for_buyer(accounts(2), None, Some(U64(1)))
            .len(),
        1
    );
    assert_eq!(
        contract
            .get_reviews_for_buyer(accounts(2), Some(U64(1)), None)
            .len(),
        0
    );
    assert_eq!(
        contract
            .get_reviews_for_buyer(accounts(2), Some(U64(1)), Some(U64(1)))
            .len(),
        0
    );
}
