/***
 * Calculates the cost of each operation in the store.
 */

use near_sdk::serde_json::json;
use near_sdk::{Gas, ONE_NEAR};

fn gas_to_near(gas: Gas) -> f64 {
    (gas.0 as f64) / (10_000_000_000_000_000.0)
}

fn storage_to_near(bytes_storage: u128) -> f64 {
    (bytes_storage as f64) / (100_000.0)
}

fn generate_text(length: usize) -> String {
    let mut text = String::new();
    for _ in 0..length {
        text.push('a');
    }
    text
}

#[tokio::test]
async fn test_methods_cost() -> anyhow::Result<()> {
    let worker = workspaces::testnet()
        .await
        .expect("Failed to start the worker");
    let account = worker.dev_create_account().await?;
    let account2 = worker.dev_create_account().await?;
    println!("----------------------------------------");

    account
        .deploy(include_bytes!("../../res/store.wasm"))
        .await?
        .into_result()?;

    let first_storage = account.view_account().await?.storage_usage;

    println!("account: {}", account.id());
    println!(
        "contract storage cost: {} NEAR",
        storage_to_near(u128::from(first_storage))
    );

    println!("--------------- INIT -------------------");
    let storage_before = account.view_account().await?.storage_usage;
    let res = account
        .call(&account.id(), "new")
        .args_json(&json!({
        "owner_id": account.id(),
        "metadata": {
            "name": generate_text(100),
            "category": 1,
            "description": generate_text(1000),
            "logo": generate_text(100),
            "cover": generate_text(100),
            "website": generate_text(100),
            "email": generate_text(100),
            "phone": generate_text(100),
            "terms": generate_text(1000),
            "tags": [generate_text(10), generate_text(10)],
            "created_at": generate_text(11),
            "updated_at": generate_text(11),
        }}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    let storage_after = account.view_account().await?.storage_usage;
    let cost = gas_to_near(Gas(res.total_gas_burnt));
    let storage = storage_after - storage_before;
    println!("gas cost: {} NEAR", cost);
    println!(
        "storage cost: {} NEAR",
        storage_to_near(u128::from(storage))
    );

    println!("--------------- CREATE ITEM ---------------");
    let storage_before = account.view_account().await?.storage_usage;
    let res = account
        .call(&account.id(), "item_create")
        .args_json(&json!({
            "metadata": {
                "title": generate_text(100),
                "description": generate_text(1000),
                "images": [generate_text(100), generate_text(100)],
                "tags": [generate_text(10), generate_text(10), generate_text(10), generate_text(10)],
            },
            "price": ONE_NEAR.to_string(),
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    let storage_after = account.view_account().await?.storage_usage;
    let item_cost = gas_to_near(Gas(res.total_gas_burnt));
    let item_storage = storage_after - storage_before;
    println!("gas cost: {} NEAR", item_cost);
    println!(
        "storage cost: {} NEAR",
        storage_to_near(u128::from(item_storage))
    );

    println!("--------------- BUY ITEM ---------------");
    let storage_before = account.view_account().await?.storage_usage;
    let res = account2
        .call(&account.id(), "item_buy")
        .args_json(&json!({
            "item_id": "0",
        }))
        .deposit(ONE_NEAR)
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    let storage_after = account.view_account().await?.storage_usage;
    let buy_cost = gas_to_near(Gas(res.total_gas_burnt));
    let buy_storage = storage_after - storage_before;
    println!("gas cost: {} NEAR", buy_cost);
    println!(
        "storage cost: {} NEAR",
        storage_to_near(u128::from(buy_storage))
    );

    println!("--------------- ORDER CANCEL ---------------");
    let storage_before = account.view_account().await?.storage_usage;
    let res = account2
        .call(&account.id(), "order_cancel")
        .args_json(&json!({
            "order_id": "0",
        }))
        .max_gas()
        .transact()
        .await?;
    println!("res: {:?}", res);
    assert!(res.is_success());
    let storage_after = account.view_account().await?.storage_usage;
    let complete_cost = gas_to_near(Gas(res.total_gas_burnt));
    let complete_storage = storage_after - storage_before;
    println!("gas cost: {} NEAR", complete_cost);
    println!(
        "storage cost: {} NEAR",
        storage_to_near(u128::from(complete_storage))
    );

    // println!("--------------- REVIEW ITEM ---------------");
    // let storage_before = account.view_account().await?.storage_usage;
    // let res = account2
    //     .call(&account.id(), "item_review")
    //     .args_json(&json!({
    //         "item_id": "0",
    //         "rating": 5,
    //         "comment": "Great work! I'll definitely order again!"
    //     }))
    //     .max_gas()
    //     .transact()
    //     .await?;
    // assert!(res.is_success());
    // let storage_after = account.view_account().await?.storage_usage;
    // let review_cost = gas_to_near(Gas(res.total_gas_burnt));
    // let review_storage = storage_after - storage_before;
    // println!("gas cost: {} NEAR", review_cost);
    // println!(
    //     "storage cost: {} NEAR",
    //     storage_to_near(u128::from(review_storage))
    // );

    // println!("---------------------------------------------------");
    // println!("--------------- STORAGE ESTIMATIONS ---------------");

    // println!(
    //     "Contract deploy: {} NEAR",
    //     storage_to_near(u128::from(first_storage))
    // );
    // println!(
    //     "10 ITEMS: {} NEAR",
    //     storage_to_near(u128::from(10 * item_storage))
    // );
    // println!(
    //     "1000 ORDERS: {} NEAR",
    //     storage_to_near(u128::from(1000 * buy_storage))
    // );
    // println!(
    //     "1000 REVIEWS: {} NEAR",
    //     storage_to_near(u128::from(1000 * review_storage))
    // );

    println!("---------------------------------------------------");

    // println!("res: {:?}", res);
    Ok(())
}

