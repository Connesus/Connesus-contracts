use crate::*;

impl Contract {
    pub fn open_donate(&mut self, account_id: &AccountId, amount: U128) {
        let prev_amount = self.donations.get(account_id).unwrap_or_default();
        let new_amount = prev_amount + amount.0;
        self.donations.insert(account_id, &new_amount);
    }
}