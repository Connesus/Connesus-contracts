use crate::*;

impl Contract {
    pub fn get_user_weight(&self, account_id: &AccountId) -> Balance {
        self.delegations.get(account_id).unwrap_or_default()
    }

    pub fn internal_delegate(&mut self, account_id: &AccountId, amount: U128) {
        let prev_amount = self
            .delegations
            .get(&account_id.to_string())
            .unwrap_or_default();
        let new_amount = prev_amount + amount.0;
        self.delegations.insert(&account_id.to_string(), &new_amount);
        self.total_delegation_amount += amount.0;
    }

    pub fn internal_undelegate(&mut self, account_id: &AccountId, amount: U128) {
        self.internal_reduce_delegation(account_id, amount);
        self.total_delegation_amount -= amount.0;
    }

    pub fn internal_reduce_delegation(&mut self, account_id: &AccountId, amount: U128) {
        let prev_amount = self.delegations.get(&account_id).unwrap_or_default();
        assert!(prev_amount >= amount.0, "ERR_INVALID_STAKING_CONTRACT");
        let new_amount = prev_amount - amount.0;
        self.delegations.insert(&account_id.to_string(), &new_amount);
    }
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleTokenContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn register_delegation(&mut self, account_id: &AccountId) {
        let community_token_id = self.community_token_id.clone();
        assert_eq!(
            env::predecessor_account_id(),
            community_token_id,
            "ERR_INVALID_CALLER"
        );
        assert_eq!(env::attached_deposit(), 16 * env::storage_byte_cost());
        self.delegations.insert(account_id, &0);
    }

    
    /// Removes given amount from given account's delegations.
    /// Returns previous, new amount of this account and total delegated amount.
    pub fn withdraw(&mut self, amount: U128) {
        let account_id: AccountId = env::predecessor_account_id();
        self.internal_undelegate(&account_id, amount);
        ext_fungible_token::ft_transfer(
            account_id.to_string(),
            amount,
            None,
            &self.community_token_id,
            ONE_YOCTO_NEAR,
            GAS_FOR_FT_TRANSFER
        );
    }
}




