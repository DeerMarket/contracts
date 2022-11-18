use std::str::FromStr;

use workspaces::{types::SecretKey, AccountId};

#[tokio::test]
async fn deploy() -> anyhow::Result<()> {
    let worker = workspaces::testnet()
        .await
        .expect("Failed to start the worker");

    // connect to the account
    let pk: SecretKey = SecretKey::from_str("put private key here")?;
    let aid: AccountId = AccountId::from_str("deerm.testnet")?;

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
