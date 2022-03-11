use crate::*;

// Bounty information.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Bounty {
    pub description: String,
    pub token: OldAccountId,
    pub total: Balance,
    pub rest: Balance,
    pub start_time: U64,
    pub duration: U64,
    pub claimer: HashMap<AccountId, Balance>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedBounty {
    Default(Bounty),
}

impl From<VersionedBounty> for Bounty {
    fn from(v: VersionedBounty) -> Self {
        match v {
            VersionedBounty::Default(b) => b,
        }
    }
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct BountyInput {
    pub description: String,
    pub token: OldAccountId,
    pub start_time: U64,
    pub duration: U64,
    pub claimer: HashMap<AccountId, Balance>,
}

impl From<BountyInput> for Bounty {
    fn from(input: BountyInput) -> Self {
        assert!(input.duration.0 > 1000000000 * 60 * 2, "DURATION_MIN_ERROR");
        let mut total = 0u128;
        for (_, value) in input.claimer.clone().into_iter() {
            total += value
        };

        let BountyInput {
            description,
            token,
            start_time,
            duration,
            claimer,
        } = input;

        Self {
            description,
            token,
            total,
            rest: total,
            start_time,
            duration,
            claimer
        }
    }
}



impl Bounty {
    pub fn claim(&mut self, account_id: &AccountId) -> Balance {
        let expired_time = self.start_time.0 + self.duration.0;
        assert!(env::block_timestamp() < expired_time, "BOUNTY_DID_NOT_EXPIRED");
        let balance_option = self.claimer.remove(account_id);
        assert!(self.claimer.remove(account_id).is_some(), "ERR_INVALID_CLAIMER");
        let balance_claimed = balance_option.unwrap_or(0);
        self.rest -= balance_claimed;
        balance_claimed
    }

    pub fn withdraw_the_rest(&mut self, receiver_id: &AccountId) {
        let expired_time = self.start_time.0 + self.duration.0;
        assert!(env::block_timestamp() > expired_time, "BOUNTY_DID_NOT_EXPIRED");
        let rest_balance = self.rest.clone();
        self.rest = 0;
        ext_fungible_token::ft_transfer(
            receiver_id.to_string(),
            rest_balance.into(),
            None,
            &self.token,
            ONE_YOCTO_NEAR,
            GAS_FOR_FT_TRANSFER
        );
    }
}

impl Contract {
    pub fn create_bounty(&mut self, bounty_input: BountyInput) -> u64 {
        let account_id = env::predecessor_account_id();
        assert_eq!(
            account_id,
            self.owner_id,
            "ONLY_OWNER"
        );
        let bounty = Bounty::from(bounty_input);
        let id = self.last_proposal_id;
        self.bounties
            .insert(&id, &VersionedBounty::Default(bounty.into()));
        self.last_proposal_id += 1;
        id
    }
}

#[near_bindgen]
impl Contract {
    pub fn with_draw_bounty_rest(&self, bounty_id: u64) {
        let account_id = env::predecessor_account_id();
        assert_eq!(
            account_id,
            self.owner_id,
            "ONLY_OWNER"
        );
        let mut bounty: Bounty = self.bounties.get(&bounty_id).expect("BOUNTY_NOT_FOUND").into();
        bounty.withdraw_the_rest(&account_id);
    }

    pub fn claim_bounty(&self,  bounty_id: u64) -> Balance {
        let account_id = env::predecessor_account_id();
        let mut bounty: Bounty = self.bounties.get(&bounty_id).expect("BOUNTY_NOT_FOUND").into();
        bounty.claim(&account_id)
    }
}