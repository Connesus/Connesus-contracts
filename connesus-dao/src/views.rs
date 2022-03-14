use std::iter::FromIterator;

use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalBaseInformation {
    pub proposer: AccountId,
    pub description: String,
    pub kind: ProposalKind,
    pub status: ProposalStatus,
    pub options: HashMap<String, VoteOption>,
    pub submission_time: U64,
    pub duration: U64,
    pub total_donations: Balance,
    pub total_delegation_amount: Balance,
    pub option_delegations: HashMap<String, Balance>
}

impl From<VersionedProposal> for ProposalBaseInformation {
    fn from(proposal: VersionedProposal) -> Self {
        let Proposal {
            proposer,
            description,
            kind,
            options,
            status,
            submission_time,
            total_delegation_amount,
            total_donations,
            option_delegations,
            duration,
            donations: _,
            votes: _
        } = proposal.into();

        Self {
            proposer,
            description,
            kind,
            options,
            status,
            submission_time,
            total_delegation_amount,
            total_donations,
            option_delegations,
            duration
        }
    }
}

// This is format of output via JSON for the proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalOutput {
    // Id of the proposal.
    pub id: u64,
    #[serde(flatten)]
    pub proposal: ProposalBaseInformation,
    pub user_select: Vote,
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BountyBaseInformation {
    pub description: String,
    pub token: OldAccountId,
    pub total: Balance,
    pub rest: Balance,
    pub start_time: U64,
    pub duration: U64,
}

impl From<VersionedBounty> for BountyBaseInformation {
    fn from(bounty: VersionedBounty) -> Self {
        let Bounty {
            description,
            token,
            total,
            rest,
            start_time,
            duration,
            claimer: _
        } = bounty.into();

        Self {
            description,
            token,
            total,
            rest,
            start_time,
            duration,
        }
    }
}

// This is format of output via JSON for the proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BountyOutput {
    // Id of the proposal.
    pub id: u64,
    pub claim_amount: Balance,
    #[serde(flatten)]
    pub bounty: BountyBaseInformation,
}

// This is format of output via JSON for the proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalDonateAsObject {
    pub account: AccountId,
    pub doate_balance: Balance
}

#[near_bindgen]
impl Contract {
    // Returns semver of this contract.
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    // Returns config of this contract.
    pub fn get_metadata(&self) -> DaoMetadata {
        self.dao_metadata.clone()
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn get_donation_balance(&self, account_id: AccountId) -> Option<Balance> {
        self.donations.get(&account_id)
    }


    // Returns staking contract if available. Otherwise returns empty.
    pub fn token_account(self) -> String {
        self.token_account
    }

    // Returns locked amount of NEAR that is used for storage.
    pub fn get_locked_storage_amount(&self) -> U128 {
        let locked_storage_amount = env::storage_byte_cost() * (env::storage_usage() as u128);
        U128(locked_storage_amount)
    }

    // Returns available amount of NEAR that can be spent (outside of amount for storage and bonds).
    pub fn get_available_amount(&self) -> U128 {
        U128(env::account_balance() - self.get_locked_storage_amount().0 - self.locked_amount)
    }

    // Returns total delegated stake.
    pub fn delegation_total_supply(&self) -> U128 {
        U128(self.total_delegation_amount)
    }

    // Returns delegated stake to given account.
    pub fn delegation_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.delegations.get(&account_id).unwrap_or_default())
    }

    // Combines balance and total amount for calling from external contracts.
    pub fn delegation_balance_ratio(&self, account_id: AccountId) -> (U128, U128) {
        (
            self.delegation_balance_of(account_id),
            self.delegation_total_supply(),
        )
    }

    // Last proposal's id.
    pub fn get_last_proposal_id(&self) -> u64 {
        self.last_proposal_id
    }
    
    pub fn get_last_bounty_id(&self) -> u64 {
        self.last_proposal_id
    }

    // Get proposals in paginated view.
    pub fn get_proposals(&self, from_index: u64, limit: u64, account_id: AccountId) -> Vec<ProposalOutput> {
        (from_index..std::cmp::min(self.last_proposal_id, from_index + limit))
            .filter_map(|id| {
                self.proposals.get(&id).map(|versioned_proposal| {
                    let proposal = Proposal::from(versioned_proposal.clone());
                    let voted = if proposal.votes.get(&account_id).is_some() {
                        proposal.votes.get(&account_id).unwrap().clone()
                    } else {
                        Vote {
                            option: "_".to_string(),
                            delegations: 0
                        }
                    };
                    ProposalOutput {
                        id,
                        proposal: ProposalBaseInformation::from(versioned_proposal.clone()),
                        user_select: voted,
                    }
                })
            })
            .collect()
    }

    // Get specific proposal.
    pub fn get_proposal(&self, id: u64, account_id: AccountId) -> Option<ProposalOutput> {
        let versioned_proposal = self.proposals.get(&id);
        let output = if versioned_proposal.clone().is_some() {
            let proposal: Proposal = Proposal::from(versioned_proposal.clone().unwrap());
            let voted = if proposal.votes.get(&account_id).is_some() {
                proposal.votes.get(&account_id).unwrap().clone()
            } else {
                Vote {
                    option: "_".to_string(),
                    delegations: 0
                }
            };
            Some(ProposalOutput {
                id,
                proposal: ProposalBaseInformation::from(versioned_proposal.unwrap().clone()),
                user_select: voted,
            })
        } else {
            None   
        };
        output
    }

    pub fn get_proposal_donation(&self, id: u64, from_index: usize, limit: usize) -> Vec<ProposalDonateAsObject> {
        let proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let donations = proposal.donations.clone();
        let mut hash_vec = Vec::from_iter(donations.into_iter());
        hash_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let donations_slice = &hash_vec[from_index..std::cmp::min(from_index + limit, hash_vec.len())];
        let response: Vec<ProposalDonateAsObject> = donations_slice.into_iter().map(|(account_id, balance)| {
            ProposalDonateAsObject {
                account: account_id.clone(),
                doate_balance: balance.clone(),
            }
        }).collect();
        response
    }

    pub fn get_bounties(&self, from_index: u64, limit: u64, account_id: AccountId) -> Vec<BountyOutput> {
        (from_index..std::cmp::min(self.last_bounty_id, from_index + limit))
            .filter_map(|id| {
                self.bounties.get(&id).map(|versioned_bounty| {
                    let bounty: Bounty = versioned_bounty.clone().into();
                    let claim_value = bounty.claimer.get(&account_id.to_string()).unwrap_or(&0u128).clone();
                    BountyOutput {
                        id,
                        claim_amount: claim_value,
                        bounty: BountyBaseInformation::from(versioned_bounty)
                    }
                })
            })
            .collect()
    }

    pub fn get_bounty(&self, id: u64, account_id: Option<AccountId>) -> Option<BountyOutput> {
        let versioned_bounty_option = self.bounties.get(&id);
        let output = if versioned_bounty_option.is_some() {
            let versioned_bounty_unwrapped = versioned_bounty_option.unwrap();
            let claim_value = if account_id.is_some() {
                let bounty: Bounty = versioned_bounty_unwrapped.clone().into();
                let claim_value = bounty.claimer.get(&account_id.unwrap()).unwrap_or(&0u128).clone();
                claim_value
            } else {
                0
            }; 
            let bounty_output = BountyOutput {
                id,
                claim_amount: claim_value,
                bounty: BountyBaseInformation::from(versioned_bounty_unwrapped)
            };
            Some(bounty_output)
        } else {
            None
        };
        output
    }
}
