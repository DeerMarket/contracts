use std::str::FromStr;
use workspaces::{types::SecretKey, AccountId};

#[tokio::test]
async fn deploy() -> anyhow::Result<()> {
    let worker = workspaces::testnet()
        .await
        .expect("Failed to start the worker");

    // connect to the account
    let pk: SecretKey = SecretKey::from_str("put private key here")?;
    let aid: AccountId = AccountId::from_str("arbiter.testnet")?;

    // let acc = worker.create_tla(aid.clone(), pk.clone()).await?.unwrap();
    // acc.clone()
    //     .delete_account(&AccountId::from_str("arbiter.testnet")?)
    //     .await?
    //     .unwrap();

    let contract = worker
        .create_tla_and_deploy(aid.clone(), pk, include_bytes!("../../res/dispute.wasm"))
        .await?
        .unwrap();

    let res = contract.call("new").max_gas().transact().await.unwrap();
    assert!(res.is_success());

    // acc.call(&aid, "whitelist")
    //     .args_json(&serde_json::json!({
    //         "account_id": "hhhhhhhhhhhhhhh.testnet"
    //     }))
    //     .max_gas()
    //     .transact()
    //     .await?
    //     .unwrap();

    println!("----------------------------------------");
    println!("Dispute contract account: {}", contract.as_account().id());
    println!("----------------------------------------");
    Ok(())
}

#[tokio::test]
async fn whitelist() -> anyhow::Result<()> {
    let worker = workspaces::testnet()
        .await
        .expect("Failed to start the worker");

    let pk: SecretKey = SecretKey::from_str("put private key here")?;
    let aid: AccountId = AccountId::from_str("arbiter.testnet")?;

    let acc = worker.create_tla(aid.clone(), pk.clone()).await?.unwrap();

    acc.call(&aid, "whitelist")
        .args_json(&serde_json::json!({
            "account_id": "arbiter.testnet"
        }))
        .max_gas()
        .transact()
        .await?
        .unwrap();

    Ok(())
}