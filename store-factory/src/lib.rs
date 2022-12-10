//! based on https://github.com/near-daos/sputnik-dao-contract/tree/41bb1481b24881d06292da0c428a2fa272414ec0/sputnikdao-factory2/src

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base58CryptoHash, Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{self, json};
use near_sdk::{env, near_bindgen, AccountId, CryptoHash, PanicOnDefault, Promise, ONE_NEAR};

mod factory_manager;
use factory_manager::FactoryManager;

type Version = [u8; 2];

// The keys used for writing data to storage via `env::storage_write`.
const DEFAULT_CODE_HASH_KEY: &[u8; 4] = b"CODE";
const FACTORY_OWNER_KEY: &[u8; 5] = b"OWNER";
const CODE_METADATA_KEY: &[u8; 8] = b"METADATA";

// The values used when writing initial data to the storage.
const CONTRACT_INITIAL_CODE: &[u8] = include_bytes!("../../res/store.wasm");
const CONTRACT_INITIAL_VERSION: Version = [0, 1];
const CONTRACT_NO_DATA: &str = "no data";

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct StoreContractMetadata {
    // version of the DAO contract code (e.g. [2, 0] -> 2.0, [3, 1] -> 3.1, [4, 0] -> 4.0)
    pub version: Version,
    // git commit id
    // representing a snapshot of the code that generated the wasm
    pub commit_id: String,
    // if available, url to the changelog to see the changes introduced in this version
    pub changelog_url: Option<String>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct StoreFactory {
    factory_manager: FactoryManager,
    stores: UnorderedSet<AccountId>,
    stores_for_creator: UnorderedMap<AccountId, UnorderedSet<AccountId>>,
}

#[near_bindgen]
impl StoreFactory {
    #[init]
    pub fn new() -> Self {
        let this = Self {
            factory_manager: FactoryManager {},
            stores: UnorderedSet::new(b"s".to_vec()),
            stores_for_creator: UnorderedMap::new(b"sc".to_vec()),
        };
        this.internal_store_initial_contract();
        this
    }

    /*********************/
    /*** Contract Code ***/
    /*********************/

    /// Store the initial contract code
    fn internal_store_initial_contract(&self) {
        self.assert_owner();
        let code = CONTRACT_INITIAL_CODE.to_vec();
        let sha256_hash = env::sha256(&code);
        env::storage_write(&sha256_hash, &code);

        self.store_contract_metadata(
            slice_to_hash(&sha256_hash),
            StoreContractMetadata {
                version: CONTRACT_INITIAL_VERSION,
                commit_id: String::from(CONTRACT_NO_DATA),
                changelog_url: None,
            },
            true,
        );
    }

    /// Change the default code hash
    pub fn set_default_code_hash(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        let code_hash: CryptoHash = code_hash.into();
        assert!(
            env::storage_has_key(&code_hash),
            "Code not found for the given code hash. Please store the code first."
        );
        env::storage_write(DEFAULT_CODE_HASH_KEY, &code_hash);
    }

    /// Delete code from storage
    pub fn delete_contract(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        self.factory_manager.delete_contract(code_hash);
        self.delete_contract_metadata(code_hash);
    }

    /// Get the default code hash
    pub fn get_default_code_hash(&self) -> Base58CryptoHash {
        slice_to_hash(&env::storage_read(DEFAULT_CODE_HASH_KEY).expect("Must have code hash"))
    }

    /// Returns non serialized code by given code hash.
    pub fn get_code(&self, code_hash: Base58CryptoHash) {
        self.factory_manager.get_code(code_hash);
    }

    /***************/
    /*** Factory ***/
    /***************/

    /// Deploy a new contract and initialize it with the given arguments.
    #[payable]
    pub fn create(&mut self, name: AccountId, args: Base64VecU8) {
        // require deposit for the storage cost
        // TODO: calculate the storage from the contract size
        let storage_cost_estimate = 3 * ONE_NEAR;
        assert!(
            env::attached_deposit() >= storage_cost_estimate,
            "Not enough attached deposit to cover storage cost"
        );
        let account_id: AccountId = format!("{}.{}", name, env::current_account_id())
            .parse()
            .unwrap();

        // add owner_id to the args argument
        let owner_id = serde_json::from_slice::<serde_json::Value>(&args.0)
            .expect("Failed to deserialize")
            .get("owner_id")
            .expect("owner_id not found")
            .as_str()
            .expect("owner_id is not a string")
            .to_string();

        let callback_args = serde_json::to_vec(&json!({
            "account_id": account_id,
            "attached_deposit": U128(env::attached_deposit()),
            "owner_account_id": owner_id
        }))
        .expect("Failed to serialize");

        self.factory_manager.create_contract(
            self.get_default_code_hash(),
            account_id,
            "new",
            &args.0,
            "on_create",
            &callback_args,
        );
    }

    /// Callback function that is called after the contract is created.
    #[private]
    pub fn on_create(
        &mut self,
        account_id: AccountId,
        attached_deposit: U128,
        owner_account_id: AccountId,
    ) -> bool {
        if near_sdk::is_promise_success() {
            self.stores.insert(&account_id);

            let mut stores_for_creator = self
                .stores_for_creator
                .get(&owner_account_id)
                .unwrap_or_else(|| {
                    UnorderedSet::new(format!("sc{}", owner_account_id).as_bytes().to_vec())
                });
            stores_for_creator.insert(&account_id);
            self.stores_for_creator
                .insert(&owner_account_id, &stores_for_creator);
            true
        } else {
            Promise::new(owner_account_id).transfer(attached_deposit.0);
            false
        }
    }

    /// Tries to update given account created by this factory to the specified code.
    pub fn update(&self, account_id: AccountId, code_hash: Base58CryptoHash) {
        let caller_id = env::predecessor_account_id();
        assert!(
            caller_id == self.get_owner() || caller_id == account_id,
            "Must be updated by the factory owner or the DAO itself"
        );
        assert!(
            self.stores.contains(&account_id),
            "Must be contract created by factory"
        );
        self.factory_manager
            .update_contract(account_id, code_hash, "update");
    }

    /// Removes the store from the factory
    pub fn remove(&mut self, store_id: AccountId, owner_id: AccountId) {
        let caller_id = env::predecessor_account_id();
        assert!(
            caller_id == self.get_owner() || caller_id == store_id,
            "Must be removed by the factory owner or the Store itself"
        );

        self.stores.remove(&store_id);

        let mut stores_for_creator = self
            .stores_for_creator
            .get(&owner_id)
            .unwrap_or_else(|| UnorderedSet::new(b"sc".to_vec()));
        stores_for_creator.remove(&store_id);

        self.stores_for_creator
            .insert(&owner_id, &stores_for_creator);
    }

    /**************/
    /*** Stores ***/
    /**************/

    /// Get all stores
    pub fn get_store_list(&self) -> Vec<AccountId> {
        self.stores.to_vec()
    }
    /// Get number of created stores.
    pub fn get_number_stores(&self) -> u64 {
        self.stores.len()
    }
    /// Get stores in paginated view.
    pub fn get_stores(&self, from_index: u64, limit: u64) -> Vec<AccountId> {
        let elements = self.stores.as_vector();
        (from_index..std::cmp::min(from_index + limit, elements.len()))
            .filter_map(|index| elements.get(index))
            .collect()
    }
    /// Get store by creator
    pub fn get_stores_by_creator(&self, creator_id: AccountId) -> Vec<AccountId> {
        self.stores_for_creator
            .get(&creator_id)
            .unwrap_or_else(|| UnorderedSet::new(b"sc".to_vec()))
            .to_vec()
    }

    /****************/
    /*** Metadata ***/
    /****************/

    pub fn store_contract_metadata(
        &self,
        code_hash: Base58CryptoHash,
        metadata: StoreContractMetadata,
        set_default: bool,
    ) {
        self.assert_owner();
        let hash: CryptoHash = code_hash.into();
        assert!(
            env::storage_has_key(&hash),
            "Code not found for the given code hash. Please store the code first."
        );

        let storage_metadata = env::storage_read(CODE_METADATA_KEY);
        if storage_metadata.is_none() {
            let mut storage_metadata: UnorderedMap<Base58CryptoHash, StoreContractMetadata> =
                UnorderedMap::new(b"m".to_vec());
            storage_metadata.insert(&code_hash, &metadata);
            let serialized_metadata =
                BorshSerialize::try_to_vec(&storage_metadata).expect("INTERNAL_FAIL");
            env::storage_write(CODE_METADATA_KEY, &serialized_metadata);
        } else {
            let storage_metadata = storage_metadata.expect("INTERNAL_FAIL");
            let mut deserialized_metadata: UnorderedMap<Base58CryptoHash, StoreContractMetadata> =
                BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
            deserialized_metadata.insert(&code_hash, &metadata);
            let serialized_metadata =
                BorshSerialize::try_to_vec(&deserialized_metadata).expect("INTERNAL_FAIL");
            env::storage_write(CODE_METADATA_KEY, &serialized_metadata);
        }

        if set_default {
            env::storage_write(DEFAULT_CODE_HASH_KEY, &hash);
        }
    }

    pub fn delete_contract_metadata(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let mut deserialized_metadata: UnorderedMap<Base58CryptoHash, StoreContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        deserialized_metadata.remove(&code_hash);
        let serialized_metadata =
            BorshSerialize::try_to_vec(&deserialized_metadata).expect("INTERNAL_FAIL");
        env::storage_write(CODE_METADATA_KEY, &serialized_metadata);
    }

    pub fn get_contracts_metadata(&self) -> Vec<(Base58CryptoHash, StoreContractMetadata)> {
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let deserialized_metadata: UnorderedMap<Base58CryptoHash, StoreContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        return deserialized_metadata.to_vec();
    }

    pub fn get_default_version(&self) -> Version {
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let deserialized_metadata: UnorderedMap<Base58CryptoHash, StoreContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        let default_metadata = deserialized_metadata
            .get(&self.get_default_code_hash())
            .expect("INTERNAL_FAIL");
        default_metadata.version
    }

    /*****************/
    /*** Ownership ***/
    /*****************/

    pub fn set_owner(&self, owner_id: AccountId) {
        self.assert_owner();
        env::storage_write(FACTORY_OWNER_KEY, owner_id.as_bytes());
    }
    pub fn get_owner(&self) -> AccountId {
        AccountId::new_unchecked(
            String::from_utf8(
                env::storage_read(FACTORY_OWNER_KEY)
                    .unwrap_or(env::current_account_id().as_bytes().to_vec()),
            )
            .expect("INTERNAL_FAIL"),
        )
    }
    fn assert_owner(&self) {
        assert_eq!(
            self.get_owner(),
            env::predecessor_account_id(),
            "Must be owner"
        );
    }
}

pub fn slice_to_hash(hash: &[u8]) -> Base58CryptoHash {
    let mut result: CryptoHash = [0; 32];
    result.copy_from_slice(&hash);
    Base58CryptoHash::from(result)
}

/// Store new contract. Non serialized argument is the contract.
/// Returns base58 of the hash of the contract.
#[no_mangle]
pub extern "C" fn store() {
    env::setup_panic_hook();
    let contract: StoreFactory = env::state_read().expect("Contract is not initialized");
    contract.assert_owner();
    let prev_storage = env::storage_usage();
    contract.factory_manager.store_contract();
    let storage_cost = (env::storage_usage() - prev_storage) as u128 * env::storage_byte_cost();
    assert!(
        storage_cost <= env::attached_deposit(),
        "Must at least deposit {} to store",
        storage_cost
    );
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, PromiseResult};

    fn to_yocto(near: &str) -> u128 {
        near.parse::<u128>().unwrap() * 10u128.pow(24)
    }

    use super::*;

    #[test]
    #[should_panic(expected = "ERR_NOT_ENOUGH_DEPOSIT")]
    fn test_create_error() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .build());
        let mut factory = StoreFactory::new();

        testing_env!(context.attached_deposit(to_yocto("2")).build());
        factory.create("test".parse().unwrap(), "{}".as_bytes().to_vec().into());
    }

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .build());
        let mut factory = StoreFactory::new();

        testing_env!(context.attached_deposit(to_yocto("3")).build());
        factory.create("test".parse().unwrap(), "{}".as_bytes().to_vec().into());

        testing_env!(
            context.predecessor_account_id(accounts(0)).build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        factory.on_create(
            format!("test.{}", accounts(0)).parse().unwrap(),
            U128(to_yocto("6")),
            accounts(0),
        );
        assert_eq!(
            factory.get_store_list(),
            vec![format!("test.{}", accounts(0)).parse().unwrap()]
        );
        assert_eq!(
            factory.get_stores(0, 100),
            vec![format!("test.{}", accounts(0)).parse().unwrap()]
        );
    }
}
