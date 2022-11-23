use std::str::FromStr;

use workspaces::{types::SecretKey, AccountId};

const PK: &str = "";
const AC: &str = "dm5.testnet";

#[tokio::test]
async fn deploy() -> anyhow::Result<()> {
    let worker = workspaces::testnet()
        .await
        .expect("Failed to start the worker");

    // connect to the account
    let pk: SecretKey = SecretKey::from_str(PK)?;
    let aid: AccountId = AccountId::from_str(AC)?;

    // let acc = worker.create_tla(aid.clone(), pk.clone()).await?.unwrap();

    // acc.delete_account(&AccountId::from_str("dmarket.testnet")?)
    //     .await?
    //     .unwrap();

    let contract = worker
        .create_tla_and_deploy(aid, pk, include_bytes!("../../res/store_factory.wasm"))
        .await?
        .unwrap();

    let res = contract.call("new").max_gas().transact().await.unwrap();

    assert!(res.is_success());

    println!("----------------------------------------");
    println!("Factory account: {}", contract.as_account().id());
    println!("----------------------------------------");
    Ok(())
}

#[tokio::test]
async fn store_new_contract() -> anyhow::Result<()> {
    let worker = workspaces::testnet()
        .await
        .expect("Failed to start the worker");

    let pk: SecretKey = SecretKey::from_str(PK)?;
    let aid: AccountId = AccountId::from_str(AC)?;

    let acc = worker.create_tla(aid.clone(), pk.clone()).await?.unwrap();

    let new_contract = include_bytes!("../../res/store.wasm");

    let res = acc
        .call(&aid, "store")
        .args(new_contract.to_vec())
        .max_gas()
        .deposit(3000000000000000000000000) // 3 NEAR
        .transact()
        .await?
        .unwrap();
    
    let res = res.json::<serde_json::Value>()?;
    let new_hash = res.as_str().unwrap();

    println!("new hash: {}", res);

    let res = acc
        .call(&aid, "set_default_code_hash")
        .args_json(&serde_json::json!({
            "code_hash": new_hash
        }))
        .max_gas()
        .transact()
        .await?
        .unwrap();
    
    println!("set_default_code_hash: {:?}", res);

    Ok(())
}
