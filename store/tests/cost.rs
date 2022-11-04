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

#[tokio::test]
async fn test_methods_cost() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
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
        "arbiter_id": "bob.near",
        "metadata": {
            "name": "Item Store",
            "description": "A store for selling items",
            "categories": ["Music", "Art"],
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
                "name": "Item 1: Emote commission (48h) - 10$ - 1 emote",
                "description": "For 10$ I'll make 1 original and costumized emote for you in only 48h! I will draw anything except mecha.Just send me a message explaining me what you want, a reference image to get an idea of how your character is or even a picture of the expression or pose you're looking for!Sizes delivered:Original emote size (800x800).Twitch sizes (112x112, 56x56 and 28x28).Custom size (you can tell me the sizes you need!)These will be sent in .png unless you need any other format or a simple background.If you have any questions feel free to send me a private message and I'll gladly reply!",
                "categories": ["Art", "Emotes"],
            },
            "price": ONE_NEAR,
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
            "item_id": 0,
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

    println!("--------------- ORDER COMPLETE ---------------");
    let storage_before = account.view_account().await?.storage_usage;
    let res = account2
        .call(&account.id(), "order_complete")
        .args_json(&json!({
            "order_id": 0,
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    let storage_after = account.view_account().await?.storage_usage;
    let complete_cost = gas_to_near(Gas(res.total_gas_burnt));
    let complete_storage = storage_after - storage_before;
    println!("gas cost: {} NEAR", complete_cost);
    println!(
        "storage cost: {} NEAR",
        storage_to_near(u128::from(complete_storage))
    );

    println!("--------------- REVIEW ITEM ---------------");
    let storage_before = account.view_account().await?.storage_usage;
    let res = account2
        .call(&account.id(), "item_review")
        .args_json(&json!({
            "item_id": 0,
            "rating": 5,
            "comment": "Great work! I'll definitely order again!"
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    let storage_after = account.view_account().await?.storage_usage;
    let review_cost = gas_to_near(Gas(res.total_gas_burnt));
    let review_storage = storage_after - storage_before;
    println!("gas cost: {} NEAR", review_cost);
    println!(
        "storage cost: {} NEAR",
        storage_to_near(u128::from(review_storage))
    );

    println!("---------------------------------------------------");
    println!("--------------- STORAGE ESTIMATIONS ---------------");

    println!(
        "Contract deploy: {} NEAR",
        storage_to_near(u128::from(first_storage))
    );
    println!(
        "10 ITEMS: {} NEAR",
        storage_to_near(u128::from(10 * item_storage))
    );
    println!(
        "1000 ORDERS: {} NEAR",
        storage_to_near(u128::from(1000 * buy_storage))
    );
    println!(
        "1000 REVIEWS: {} NEAR",
        storage_to_near(u128::from(1000 * review_storage))
    );

    println!("---------------------------------------------------");

    // println!("res: {:?}", res);
    Ok(())
}
