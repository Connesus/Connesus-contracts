use crate::*;

pub(crate) fn assert_account_id(account_id: &AccountId) {
    assert_eq!(
        env::predecessor_account_id(), 
        account_id.clone(),
        "ERR_ACCOUNT_ID_NOT_ALLOWED"
    )
}