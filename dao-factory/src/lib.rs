use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::env::STORAGE_PRICE_PER_BYTE;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise,
};

const DAO_WASM_CODE: &[u8] = include_bytes!("../../out/connesus-dao.wasm");

const EXTRA_BYTES: usize = 10000;
const GAS: Gas = 50_000_000_000_000;
type daoId = String;

pub fn is_valid_dao_id(dao_id: &daoId) -> bool {
    for c in dao_id.as_bytes() {
        match c {
            b'0'..=b'9' | b'a'..=b'z' => (),
            _ => return false,
        }
    }
    true
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Daos,
    StorageDeposits,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct DaoFactory {
    pub daos: UnorderedMap<AccountId, DaoArgs>,
    pub storage_deposits: LookupMap<AccountId, Balance>,
    pub storage_balance_cost: Balance,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoMetadata {
    // Name of the DAO.
    pub name: String,
    // Purpose of this DAO.
    pub purpose: String,
    // Generic metadata. Can be used by specific UI to store additional data.
    // This is not used by anything in the contract.
    pub thumbnail: String,

    pub symbol: String,

    pub facebook: Option<String>,

    pub youtube: Option<String>,

    pub twitter: Option<String>,

    pub discord: Option<String>,

    pub instagram: Option<String>,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DaoArgs {
    dao_contract_id: AccountId,
    metadata: DaoMetadata,
}

#[near_bindgen]
impl DaoFactory {
    #[init]
    pub fn new() -> Self {
        let mut storage_deposits = LookupMap::new(StorageKey::StorageDeposits);

        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = "a".repeat(64);
        storage_deposits.insert(&tmp_account_id, &0);
        let storage_balance_cost =
            Balance::from(env::storage_usage() - initial_storage_usage) * STORAGE_PRICE_PER_BYTE;
        storage_deposits.remove(&tmp_account_id);

        Self {
            daos: UnorderedMap::new(StorageKey::Daos),
            storage_deposits,
            storage_balance_cost,
        }
    }

    fn get_min_attached_balance(&self, args: &DaoArgs) -> u128 {
        ((DAO_WASM_CODE.len() + EXTRA_BYTES + args.try_to_vec().unwrap().len() * 2) as Balance
            * STORAGE_PRICE_PER_BYTE)
            .into()
    }

    pub fn get_required_deposit(&self, args: DaoArgs, account_id: ValidAccountId) -> U128 {
        let args_deposit = self.get_min_attached_balance(&args);
        if let Some(previous_balance) = self.storage_deposits.get(account_id.as_ref()) {
            args_deposit.saturating_sub(previous_balance).into()
        } else {
            (self.storage_balance_cost + args_deposit).into()
        }
    }

    #[payable]
    pub fn storage_deposit(&mut self) {
        let account_id = env::predecessor_account_id();
        let deposit = env::attached_deposit();
        if let Some(previous_balance) = self.storage_deposits.get(&account_id) {
            self.storage_deposits
                .insert(&account_id, &(previous_balance + deposit));
        } else {
            assert!(deposit >= self.storage_balance_cost, "Deposit is too low");
            self.storage_deposits
                .insert(&account_id, &(deposit - self.storage_balance_cost));
        }
    }

    pub fn get_number_of_daos(&self) -> u64 {
        self.daos.len()
    }

    pub fn get_daos(&self, from_index: u64, limit: u64) -> Vec<DaoArgs> {
        let daos = self.daos.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, daos.len()))
            .filter_map(|index| daos.get(index))
            .collect()
    }

    pub fn get_dao(&self, dao_id: AccountId) -> Option<DaoArgs> {
        self.daos.get(&dao_id)
    }

    #[payable]
    pub fn create_dao(&mut self, args: DaoArgs) -> Promise {
        if env::attached_deposit() > 0 {
            self.storage_deposit();
        }
        let dao_id = args.metadata.symbol.to_ascii_lowercase();
        assert!(is_valid_dao_id(&dao_id), "Invalid Symbol");
        let dao_account_id = format!("{}.{}", dao_id, env::current_account_id());
        assert!(
            env::is_valid_account_id(dao_account_id.as_bytes()),
            "dao Account ID is invalid"
        );

        let account_id = env::predecessor_account_id();

        let required_balance = self.get_min_attached_balance(&args);
        let user_balance = self.storage_deposits.get(&account_id).unwrap_or(0);
        assert!(
            user_balance >= required_balance,
            "Not enough required balance"
        );
        self.storage_deposits
            .insert(&account_id, &(user_balance - required_balance));

        let initial_storage_usage = env::storage_usage();

        assert!(
            self.daos.insert(&dao_id, &args).is_none(),
            "dao ID is already taken {}", dao_account_id
        );

        let storage_balance_used =
            Balance::from(env::storage_usage() - initial_storage_usage) * STORAGE_PRICE_PER_BYTE;

        Promise::new(dao_account_id)
            .create_account()
            .transfer(required_balance - storage_balance_used)
            .deploy_contract(DAO_WASM_CODE.to_vec())
            .function_call(b"new".to_vec(), serde_json::to_vec(&args).unwrap(), 0, GAS)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::{testing_env, env, Balance};
    use near_sdk::MockedBlockchain;

    const MINT_STORAGE_COST: u128 = 58700000000000000000000;
    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;
    
}
