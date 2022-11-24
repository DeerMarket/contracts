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
        .args_json(&serde_json::json!({ "code_hash": new_hash }))
        .max_gas()
        .transact()
        .await?
        .unwrap();

    println!("set_default_code_hash: {:?}", res);

    Ok(())
}
