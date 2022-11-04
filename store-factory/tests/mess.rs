use near_sdk::{
    env::sha256_array,
    json_types::{Base58CryptoHash, Base64VecU8},
    serde_json::json,
    CryptoHash,
};
use workspaces::{Account, Contract};

// helpers
async fn h_init() -> (Contract, Account, Account) {
    println!("----------------------------------------");
    let worker = workspaces::sandbox()
        .await
        .expect("Failed to start the worker");

    let contract = worker
        .dev_deploy(include_bytes!("../../res/store_factory.wasm"))
        .await
        .unwrap();
    println!("Factory account: {}", contract.as_account().id());

    let alice = worker.dev_create_account().await.unwrap();
    let bob = worker.dev_create_account().await.unwrap();

    let res = contract.call("new").max_gas().transact().await.unwrap();

    assert!(res.is_success());

    println!("----------------------------------------");
    (contract, alice, bob)
}
fn slice_to_hash(hash: &[u8]) -> Base58CryptoHash {
    let mut result: CryptoHash = [0; 32];
    result.copy_from_slice(&hash);
    Base58CryptoHash::from(result)
}

#[tokio::test]
async fn test_code_management() -> anyhow::Result<()> {
    let (contract, _, _) = h_init().await;

    // 1. Store a new code
    let code = include_bytes!("../../res/fixed_price_contract.wasm");
    let code_hash_array = sha256_array(code);
    let code_hash = slice_to_hash(&code_hash_array);
    let code = code.to_vec();

    let res = contract
        .call("store")
        .args(code.clone())
        .max_gas()
        .deposit(1430160000000000000000000)
        .transact()
        .await?;
    assert!(res.is_success());

    // 2. Check that the code stored is the same as the one we sent
    let res = contract
        .call("get_code")
        .args_json(json!({
            "code_hash": code_hash,
        }))
        .max_gas()
        .view()
        .await?;
    assert_eq!(code, res.result);

    // 3. Set the new code as the default one
    let res = contract
        .call("set_default_code_hash")
        .args_json(json!({
            "code_hash": code_hash,
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // 4. Check that the default code is the one we set
    let res = contract.view("get_default_code_hash", vec![]).await?;
    let res = serde_json::from_slice(&res.result)?;
    assert_eq!(code_hash, res);

    // 5. Remove the code
    let res = contract
        .call("delete_contract")
        .args_json(json!({
            "code_hash": code_hash,
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // 6. Check that the code is removed
    let res = contract
        .call("get_code")
        .args_json(json!({
            "code_hash": code_hash,
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_failure(), "Code should be removed");

    Ok(())
}

#[tokio::test]
async fn test_factory_basics() -> anyhow::Result<()> {
    let (contract, alice, _) = h_init().await;

    let contract_id = contract.as_account().id();

    // 1. Create a new contract
    let args: String = json!({
    "owner_id": alice.id(),
    "arbiter_id": "bob.near",
    "metadata": {
        "name": "Gig Store",
        "description": "A store for selling gigs",
        "image": "https://example.com/gig-store.png"
    }})
    .to_string();
    let args = Base64VecU8::from(args.as_bytes().to_vec());
    let res = alice
        .call(&contract_id, "create")
        .args_json(&json!({
            "name": "gamer",
            "args": args,
        }))
        .max_gas()
        .deposit(2217040000000000000000000)
        .transact()
        .await?;

    println!("gas used: {}", res.total_gas_burnt);
    assert!(res.is_success());
    let store_contract_id = format!("gamer.{}", contract_id);

    // 2. Check that the contract was created
    let res = contract
        .call("get_stores")
        .args_json(json!({
            "from_index": 0,
            "limit": 1,
        }))
        .max_gas()
        .view()
        .await?;
    let res: Vec<String> = serde_json::from_slice(&res.result)?;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], store_contract_id);

    // 3. try to create a contract with the same name
    let res = alice
        .call(&contract_id, "create")
        .args_json(&json!({
            "name": "gamer",
            "args": args,
        }))
        .max_gas()
        .deposit(2217040000000000000000000)
        .transact()
        .await?;
    assert!(res.is_failure());

    Ok(())
}
