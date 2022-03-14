use crate::*;

// Account ID used for $NEAR in near-sdk v3.
// Need to keep it around for backward compatibility.
pub const OLD_BASE_TOKEN: &str = "";

// Account ID that represents a token in near-sdk v3.
// Need to keep it around for backward compatibility.
pub type OldAccountId = String;

// 1 yN to prevent access key fraud.
pub const ONE_YOCTO_NEAR: Balance = 1;

// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;

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

    pub twitter: Option<String>,

    pub discord: Option<String>,

    pub instagram: Option<String>,
}

#[cfg(test)]
impl DaoMetadata {
    pub fn test_config() -> Self {
        Self {
            name: "Test".to_string(),
            purpose: "to test".to_string(),
            thumbnail: "".to_string(),
        }
    }
}

// Set of possible action to take.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    Vote {option_id: String},
    Finalize
}

