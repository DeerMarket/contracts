#[tokio::test]
async fn deploy() -> anyhow::Result<()> {
    let worker = workspaces::testnet()
        .await
        .expect("Failed to start the worker");
    let contract = worker
        .dev_deploy(include_bytes!("../../res/store_factory.wasm"))
        .await
        .unwrap();
    let res = contract.call("new").max_gas().transact().await.unwrap();
    assert!(res.is_success());

    println!("----------------------------------------");
    println!("Factory account: {}", contract.as_account().id());
    println!("----------------------------------------");
    Ok(())
}
